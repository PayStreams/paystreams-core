

## Things to do 

+ Rework this repo into a rust workspace as it will have multiple contracts and a package with common code 
+ Review the basic flow, we are compliant with cw-1620 already so we can start to customize a bit 
+ Initial use case is streaming axlUSDC. This is assumed to be a native token everywhere we deploy so it covers that use case. 
+ Streaming of CW20, most of the support is there in the contract or at least can be added as its just receive cw20 and some if statements but it should be added early to enable all tokens 
+ Streaming of CW721, this is a bit more complex as we need to track the NFTs and also have a way to transfer them. This is a good use case for a hook system.
+ Setup Gas Profiling early on so we can do testing by unit, integration and also get an idea of gas costs to see if they can be optimized 
+ Setup CI/CD pipeline for testing and deployment
+ Review a hub model whereby one hub contract can deploy streams, this is an evolution of the current model, ideally we avoid using a second contract but it may be needed to enable some features, if we can avoid it we should. 
+ Enable creation of streams across chains, this can be done from the hub contract on its host chain and it will deploy the stream on the target chain using a target contract which understands the send IBC command from the host and can then spin up a stream with the received funds on that chain. 
+ Stop here and review entire design, should we stick with 1 hub using IBC or go with multi chains. Migaloo will be the home of the hub. 



## Design 
This should be moved to a design.md maybe 

What we want to achieve and maintain is a single contract approach to doing streaming payments. This gives us immediate benefits in terms of gas costs and also allows us to have a single contract that can be used by multiple parties. Where we want to do customizations we can build a hook system or another contract which can act as the start of a plugin system.

Its worth stating again that streams are very similar to vesting schedules that are present in so many systems with staking.
We are using this concept in a new way to enable streaming payments which is a very powerful concept but can just be another term for a vesting schedule.
This is where simplicity of design and ability to integrate is key. 

We will use curve utilities provided by Wynd to enable creation of streams based on a bonding curve which enables a number of presets we can offer. 