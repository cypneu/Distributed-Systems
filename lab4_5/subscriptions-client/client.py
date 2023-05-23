import asyncio
import os
from io import TextIOWrapper

import grpc

from proto.subscription_pb2 import Notification, SubscriptionRequest
from proto.subscription_pb2_grpc import SubscriptionStub


async def async_input(prompt: str) -> str:
    print(prompt, end="", flush=True)
    return await asyncio.get_event_loop().run_in_executor(None, input)


async def subscribe(
    subscriptions: list[str], stub: SubscriptionStub, file: TextIOWrapper
) -> None:
    subscriptions = list(
        map(
            lambda item: SubscriptionRequest(
                name=item[0], notification_frequency=int(item[1])
            ),
            subscriptions,
        )
    )

    stream = stub.Subscribe(subscriptions)

    try:
        async for notification in stream:
            notification_message = (
                f"Notification frequency: {notification.frequency} s.\n"
                f"Notification type: {Notification.NotificationType.Name(notification.notification_type)}\n"
                f"Content: {notification.content.body}\n\n"
            )

            file.write(notification_message)
            file.flush()
    except grpc.aio.AioRpcError:
        pass


async def main() -> None:
    background_tasks = []

    os.makedirs("out", exist_ok=True)
    with open("out/output.txt", mode="w") as output_file:

        channel_options = [
            ("grpc.keepalive_time_ms", 30000),  # send keepalive ping every 30 seconds
            ("grpc.keepalive_timeout_ms", 20000),  # wait 20 seconds for keepalive ack
            ("grpc.keepalive_permit_without_calls", True),
        ]

        async with grpc.aio.insecure_channel(
            "[::1]:50051", options=channel_options
        ) as channel:
            stub = SubscriptionStub(channel)

            while True:
                user_input = await async_input("Enter subscription: ")
                if user_input == "exit":
                    break

                subscriptions = user_input.split(", ")
                subscriptions = list(map(lambda item: item.split(), subscriptions))
                if any(len(sub) != 2 for sub in subscriptions):
                    continue

                task = asyncio.create_task(subscribe(subscriptions, stub, output_file))
                background_tasks.append(task)


if __name__ == "__main__":
    asyncio.run(main())
