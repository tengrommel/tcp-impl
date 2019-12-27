#!/bin/bash
cargo b
sudo setcap cap_net_admin=eip target/debug/untitled
target/debug/untitled &
pid=$!
sudo ip addr add 10.0.0.1/24 dev tun0
sudo ip link set up dev tun0
trap "kill $pid" TERM
wait $pid