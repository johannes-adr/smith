# Smith
<sup>Schematic Modulation to Increase Throughput _</sup>

<img src="https://i.pinimg.com/originals/e0/a6/b4/e0a6b4e8967de70c02480da9c45b6368.jpg" width="100">

## Abstract
This Rust crate provides functionality for serializing and deserializing a given data structure into a binary format based on a specified schema. The crate is designed to be easy to use, fast and can be integrated into any Rust project with minimal effort.

## Schema
Example of schema file ("example.schema.bdr"):
```rust
enum Optional<T>{
    Some(T)
    None
}

enum Result<A,B>{
    Ok(A)
    Err(B)
}

struct Order{
    table_number: udInt
    items: Array<string>
}

struct Packet<T>{
    id: udInt
    payload: T
}

enum ErrorType{
    MalformedBinary
}

enum ServerPayload{
    Ack
    Error(ErrorType)
}

//Only for creating generic versions for Packet (generic declarations needs an non_generic declaration to 
//depend on it)
struct Root{
    b: Packet<ServerPayload>
}
```