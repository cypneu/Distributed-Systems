FROM python:3.11

RUN apt-get update
RUN apt-get install -y apt-file
RUN apt-file update
RUN apt-get install -y python3-zeroc-ice zeroc-ice-compilers vim

RUN pip install pipenv --no-cache-dir

WORKDIR /app

COPY Pipfile Pipfile.lock ./

RUN pipenv install --system --deploy

COPY . .

RUN slice2py slice/greet.ice
