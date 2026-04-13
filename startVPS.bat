@echo off
ssh -N ^
  -i "C:\Users\kvxshenyjbetxn\.ssh\soloveyko-tunnel-nopass" ^
  -o ServerAliveInterval=30 ^
  -o ServerAliveCountMax=3 ^
  -o ExitOnForwardFailure=yes ^
  -R 127.0.0.1:39245:127.0.0.1:39245 ^
  openclawops@195.189.227.103
