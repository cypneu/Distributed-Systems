import numpy as np
import ray
import requests
import torch
import torch.nn as nn
import torch.nn.functional as F
import torch.optim as optim
import torchvision
import torchvision.transforms as transforms
from PIL.Image import Image
from ray import serve, train
from ray.air import session
from ray.air.config import ScalingConfig
from ray.data.preprocessors import TorchVisionPreprocessor
from ray.serve import PredictorDeployment
from ray.serve.http_adapters import json_to_ndarray
from ray.train.batch_predictor import BatchPredictor
from ray.train.torch import TorchCheckpoint, TorchPredictor, TorchTrainer

ray.init()

train_dataset = torchvision.datasets.CIFAR10("data", download=True, train=True)
test_dataset = torchvision.datasets.CIFAR10("data", download=True, train=False)

train_dataset: ray.data.Dataset = ray.data.from_torch(train_dataset)
test_dataset: ray.data.Dataset = ray.data.from_torch(test_dataset)


def convert_batch_to_numpy(batch: tuple[Image, int]) -> dict[str, np.ndarray]:
    images = np.stack([np.array(image) for image, _ in batch])
    labels = np.array([label for _, label in batch])
    return {"image": images, "label": labels}


train_dataset = train_dataset.map_batches(convert_batch_to_numpy).fully_executed()
test_dataset = test_dataset.map_batches(convert_batch_to_numpy).fully_executed()


class Net(nn.Module):
    def __init__(self):
        super().__init__()
        self.conv1 = nn.Conv2d(3, 6, 5)
        self.pool = nn.MaxPool2d(2, 2)
        self.conv2 = nn.Conv2d(6, 16, 5)
        self.fc1 = nn.Linear(16 * 5 * 5, 120)
        self.fc2 = nn.Linear(120, 84)
        self.fc3 = nn.Linear(84, 10)

    def forward(self, x):
        x = self.pool(F.relu(self.conv1(x)))
        x = self.pool(F.relu(self.conv2(x)))
        x = torch.flatten(x, 1)  # flatten all dimensions except batch
        x = F.relu(self.fc1(x))
        x = F.relu(self.fc2(x))
        x = self.fc3(x)
        return x


def train_loop_per_worker(config):
    model = train.torch.prepare_model(Net())

    criterion = nn.CrossEntropyLoss()
    optimizer = optim.SGD(model.parameters(), lr=0.001, momentum=0.9)

    train_dataset_shard = session.get_dataset_shard("train")

    for epoch in range(2):
        running_loss = 0.0
        train_dataset_batches = train_dataset_shard.iter_torch_batches(
            batch_size=config["batch_size"]
        )
        for i, batch in enumerate(train_dataset_batches):
            # get the inputs and labels
            inputs, labels = batch["image"], batch["label"]

            # zero the parameter gradients
            optimizer.zero_grad()

            # forward + backward + optimize
            outputs = model(inputs)
            loss = criterion(outputs, labels)
            loss.backward()
            optimizer.step()

            # print statistics
            running_loss += loss.item()
            if i % 2000 == 1999:  # print every 2000 mini-batches
                print(f"[{epoch + 1}, {i + 1:5d}] loss: {running_loss / 2000:.3f}")
                running_loss = 0.0

        metrics = dict(running_loss=running_loss)
        checkpoint = TorchCheckpoint.from_state_dict(model.state_dict())
        session.report(metrics, checkpoint=checkpoint)


transform = transforms.Compose(
    [transforms.ToTensor(), transforms.Normalize((0.5, 0.5, 0.5), (0.5, 0.5, 0.5))]
)
preprocessor = TorchVisionPreprocessor(columns=["image"], transform=transform)


trainer = TorchTrainer(
    train_loop_per_worker=train_loop_per_worker,
    train_loop_config={"batch_size": 2},
    datasets={"train": train_dataset},
    scaling_config=ScalingConfig(num_workers=len(ray.nodes())),
    preprocessor=preprocessor,
)
result = trainer.fit()
latest_checkpoint = result.checkpoint


batch_predictor = BatchPredictor.from_checkpoint(
    checkpoint=latest_checkpoint,
    predictor_cls=TorchPredictor,
    model=Net(),
)

outputs: ray.data.Dataset = batch_predictor.predict(
    data=test_dataset,
    dtype=torch.float,
    feature_columns=["image"],
    keep_columns=["label"],
)


def convert_logits_to_classes(df):
    best_class = df["predictions"].map(lambda x: x.argmax())
    df["prediction"] = best_class
    return df[["prediction", "label"]]


predictions = outputs.map_batches(convert_logits_to_classes)

print(predictions.show(1))


def calculate_prediction_scores(df):
    df["correct"] = df["prediction"] == df["label"]
    return df


scores = predictions.map_batches(calculate_prediction_scores)

print(scores.show(1))
print(scores.sum(on="correct") / scores.count())


serve.run(
    PredictorDeployment.bind(
        TorchPredictor,
        latest_checkpoint,
        model=Net(),
        http_adapter=json_to_ndarray,
    ),
    host="0.0.0.0",
)

image = test_dataset.take(2)[1]["image"]
payload = {"array": image.tolist(), "dtype": "float32"}
response = requests.post("http://0.0.0.0:8000/", json=payload)

labels = [
    "airplane",
    "automobile",
    "bird",
    "cat",
    "deer",
    "dog",
    "frog",
    "horse",
    "ship",
    "truck",
]
print(
    "\n\nPredicted class:",
    labels[np.argmax(response.json()["predictions"])],
    end="\n\n\n",
)
