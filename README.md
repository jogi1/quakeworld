quakeworld
=====
A rust library for working with quakeworld. 
### available features
 * protocol
   * [quakeworld::protocol::message::Message](./src/protocol/message.rs) - reading data types from a byte array
   * [quakeworld::protocol::types](./src/protocol/types.rs) - data types

 * mvd
   * [quakeworld::mvd::Mvd](./src/mvd/mod.rs) - parsing mvd file format

 * state
   * [quakeworld::state::State](./src/state/mod.rs) - using Message types to create a game state

 * utils 
   * [quakeworld::utils::AsciiConverter](./src/utils/ascii_converter.rs) - converting byte arrays to printable ascii

 * ascii_strings - when reading strings they will be converted to printable ascii, original bytes are also being kept see [here](./src/protocol/types.rs#L12)

Features that are enabled by default are protocol, mvd, and util.  
Everything is serializable via [serde](https://github.com/serde-rs/serde) (json,...). Supports wasm as target ('it compiles' ```cargo build --target wasm32-unknown-unknown```) 

### Goals 
probably in order of being implemented too
* network - client
* qwd - qwd format parsing
* mvd - creating a mvd from states

### Documentation
could be better, aka non existing at the moment

### Example
  * [minimal mvd parser](./examples/mvd_parser.rs)
  * [more elaborate mvd parser](https://github.com/jogi1/statyr) soon™