# cw-nfts-simple

cw-nfts-simple-base is a base contract for easy extension but not compilable itself. <br>
It is based on function-oriented style different from original cw721-base. <br>

This repository contains:
* packages/cw721-simple-base: base codes to extend custom cw721 based nfts <br>
* contracts/*: example contracts using cw721-simple

contracts under contracts directory covers two approaches to extend base contract easily: <br> 
1. Implement Custom messages whose entry point has generic parameters -> cw721-simple-metadata
2. Wrap base messages with user-defined message -> cw721-simple-metadata-without-custom-msg

Also, converting into owned type message could be another solution for extension: <br>
https://github.com/D3LAB-DAO/cosmonaut-contract/blob/main/contracts/cosmonaut-cw20/src/msg.rs