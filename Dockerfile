FROM backpackapp/build:v0.30.0

WORKDIR /workspace
COPY . .

RUN anchor build
