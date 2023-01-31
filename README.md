# Fedimint Smol File System

This module aims to solve issues of how to store small files that have mission critical data and must be available at all times. It also aims to be a way to add revenue streams to any fedimint!

This problem comes up in many kinds of software, for example lightning nodes that have to store state, exotic multisig setups that have to store XPUBs, and many more!

SmolFS solves this problem by allowing users to submit an identifier and a binary blob. The identifier should ideally be a public key, where the seed is safely backed up. The binary blob could be the plaintext itself, however it should ideally be encrypted. A good scheme is to generate a mnemonic, get a public key, then encrypt the data with the private key. 

SmolFS works by having this KV store sit right next to the other transactions that the mint stores, so the availablitity of the data should be as good as the availablitity of the mints transactions.

It's called smolfs beacuse its meant to be used for small files, things like config files or notes. Ideally, making a client for this should be as easy as importing a library and calling one or two functions. The user would also need to have a prefered instance of a mint.

To see an example of the code, you need to run the test, which can be done by calling "create_output_for_smolfs()".

You'll see that it stores a KV, does a consensus round, then you are able to retrieve the data.

My goals for this module are twofold;
- make a useful tool for others
- make the simplest useful app, that shows the basics of how to interact with fedimint for other fedimint module devs.

By finishing the client code, I can get the first part done. By cloning the repo and making a tutorial video, I can finish the second one.

My plan is to also start a youtube channel/podcast called "Fedimint Radio" which at first would be a place for devs to learn about SysAdmin, Rust, Bitcoin and Nix. However, as fedimint grows in adoption, I would also like to interview users and guardians of fedimint. So that I can learn about their challenges and opportunities.

Still so much to do with fedimint! It's been a fantastic experience for me.
