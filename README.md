# BananaFarm

## A simulated, distributed soil-moisture data collection system

### About

The point of this project was for me to learn more about distributed systems and IoT. I knew I wanted to write something with rumqtt and this bananafarm automated irrigation system was what I came up with lol.

### Architecture

The system uses mqtt for communation between the publishers, which read data from the soil-moisture sensors, and the subscribers, which write the data to the time series database.

The mqtt clients are built using the rumqttc library, since I wanted to do something with rust.

The whole system is deployed on k3s to enable a full ci/cd pipeline.

There is currently no frontend for this system but I plan on creating one in the near-future, maybe.
