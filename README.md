## ConCoin
A cryptocurrency with provable randomness. This client formats input. The [backend](https://github.com/CPoof/ConCoin) provides the security.

## Requirements
Unlike the backend, the client only needs Rust.

## Performance Note
The source code has native-cpu enabled. This allows for improved performance on your machine. \
If you intend to build an executable, consider removing this in .cargo/config.toml.

## Installation
Download the latest [release](https://github.com/CPoof/ConCoin/releases) of ConCoin-Client. 
After installing the source code, change directory to subfolder, program. 
```
cd sourcecodename
cd program
```
You can now run or build as normal: 
```
cargo run --release
cargo run --build
```
## Usage as a Client for ConCoin
1. Enter an input
2. The resulting hash is a combination of BOTH the input and a pepper \
This ensures a random hash.
4. Save the secret values to a json file with "s"\
You can personally encrypt the file if neccesary.
5. You can now give the original input to your commited hash. 
