

## Things to do 

+ Rework this repo into a rust workspace as it will have multiple contracts and a package with common code 
+ Review the basic flow, we are compliant with cw-1620 already so we can start to customize a bit 
+ Setup Gas Profiling early on so we can do testing by unit, integration and also get an idea of gas costs to see if they can be optimized 
+ Setup CI/CD pipeline for testing and deployment


## Design 
This should be moved to a design.md maybe 

What we want to achieve and maintain is a single contract approach to doing streaming payments. This gives us immediate benefits in terms of gas costs and also allows us to have a single contract that can be used by multiple parties. Where we want to do customizations we can build a hook system or another contract which can act as the start of a plugin system.

Its worth stating again that streams are very similar to vesting schedules that are present in so many systems with staking.
We are using this concept in a new way to enable streaming payments which is a very powerful concept but can just be another term for a vesting schedule.
This is where simplicity of design and ability to integrate is key. 

We will use curve utilities provided by Wynd to enable creation of streams based on a bonding curve which enables a number of presets we can offer. 