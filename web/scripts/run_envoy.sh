docker run -i --rm --name envoy --net=host -v $PWD/envoy.yml:/etc/envoy/envoy.yaml envoyproxy/envoy-dev
