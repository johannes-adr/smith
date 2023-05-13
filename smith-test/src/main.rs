use std::{fs, time::{Instant, Duration}, ops::Add};

use Types::Root;
use byte_unit::Byte;
use protobuf::Message;
use protos::schema::SomeMessage;
use smith_core::Smith;
mod protos;
fn main() {
    let json =  fs::read_to_string("./smith-test/example.json").unwrap();
    let jsonmin =  fs::read_to_string("./smith-test/example.min.json").unwrap();
    let lenp =test_proto_size() as u128;
    let lens =test_smith_size() as u128;

    println!("json: {} (minified: {}) proto: {} smith: {}",
    Byte::from_bytes(json.len() as u128).get_appropriate_unit(false).to_string(),
    Byte::from_bytes(jsonmin.len() as u128).get_appropriate_unit(false).to_string(),
    Byte::from_bytes(lenp).get_appropriate_unit(false).to_string(),
    Byte::from_bytes(lens).get_appropriate_unit(false).to_string());
    let runs = 100;

    let smith = Smith::new(&fs::read_to_string("./smith-test/schema.smith").unwrap());
    let typ = smith.get_type("Root").unwrap();

    let smith_bytes = fs::read("./smith-test/out.bin").unwrap();
    let root: Root = smith.binary2rust(&smith_bytes, &typ).unwrap(); 
    let jsonval = serde_json::to_value(&json).unwrap();
    let proto_root = smithroot2proto(root.clone());
    let mut proto_bytes = Vec::new();
    
    bench("SerdeJson serialize (static)",runs, ||{
        _=serde_json::to_string(&root);
    });
    bench("SerdeJson serialize (dynamic)",runs, ||{
        _=serde_json::to_string(&jsonval);
    });

    bench("Proto serialize",runs, ||{
        proto_bytes = Vec::new();
        proto_root.write_to_vec(&mut proto_bytes).unwrap();
    });
    bench("Smith serialize",runs, ||{
        _=smith.rust2binary(&root, &typ);
    });

    bench("SerdeJson deserialize (dynamic - min)",runs, ||{
        _=serde_json::to_value(&jsonmin);
    });

    bench("SerdeJson deserialize (static - min)",runs, ||{
        _=serde_json::from_str::<Root>(&jsonmin);
    });
    bench("Proto deserialize", runs, ||{
        _=SomeMessage::parse_from_bytes(&proto_bytes).unwrap();
    });
    bench("Smith deserialize", runs, ||{
        smith.binary2rust::<Root>(&smith_bytes, &typ).unwrap();
    });
}

#[inline(always)]
fn bench(name: &str,runs: u32,mut func: impl FnMut()){
    let mut t = Duration::new(0, 0);
    for _ in 0..runs{
        let start = Instant::now();
        func();
        t = t.add(start.elapsed());
    }
    println!("{name} took {:?}", t/runs);
}

fn smithroot2proto(root: Root) -> SomeMessage{
    let example = protos::schema::SomeMessage::new();
    
    //convert root to some message
    let mut new_root = protos::schema::SomeMessage::new();
    for person in root.clone().person{
        let mut new_person = protos::schema::some_message::Person::new();
        new_person.age = person.age as u32;
        new_person.index = person.index as u32;
        new_person.alive = person.alive;
        new_person.isActive = person.isActive;
        new_person.picture = person.picture.to_string();
        new_person.name = person.name;
        new_person.latitude = person.latitude;
        new_person.longitude = person.longitude;
        for friend in person.friends{
            let mut new_friend = protos::schema::some_message::Friends::new();
            new_friend.id = friend.id as u32;
            new_friend.name = friend.name.to_string();
            new_person.friends.push(new_friend);
        }
        new_root.person.push(new_person);
    }
    new_root
}

fn test_proto_size() -> usize{
    let (_,root) = read_outbin();
    let mut bytes = Vec::new();
    smithroot2proto(root).write_to_vec(&mut bytes).unwrap();
    return bytes.len();
}

fn test_smith_size() -> usize{
    let smith = Smith::new(&fs::read_to_string("./smith-test/schema.smith").unwrap());
    let typ = smith.get_type("Root").unwrap();
    let (bytes,root) = read_outbin();
    let new = smith.rust2binary(&root, &typ).unwrap();
    return new.len();
}

fn read_outbin() -> (&'static [u8],Types::Root<'static>){
    let smith = Box::leak(Box::new(Smith::new(&fs::read_to_string("./smith-test/schema.smith").unwrap())));
    let typ = Box::leak(Box::new(smith.get_type("Root").unwrap()));
    let bytes = fs::read("./smith-test/out.bin").unwrap().leak();
    let mut back: Types::Root = smith.binary2rust(bytes, typ).unwrap();
    return (bytes, back);
}






fn testsmith2(){
    let smith = Smith::new(&fs::read_to_string("./schema.smith").unwrap());
    let typ = smith.get_type("Root").unwrap();
    let json = fs::read_to_string("./example.json").unwrap();

    let bytes = smith.json2binary(&json, &typ).unwrap();
    println!(
        "Old: {}, new: {}, Saved space: {}%",
        Byte::from_bytes(json.len() as u128).get_appropriate_unit(false).to_string(),
        Byte::from_bytes(bytes.len() as u128).get_appropriate_unit(false).to_string(),
        (1.0 - bytes.len() as f32 / json.len() as f32) * 100.0
    );
     //let bytes = fs::read("./out.bin").unwrap();
    let mut back: Types::Root = smith.binary2rust(&bytes, &typ).unwrap();
    // for i in 0..1{
    //     back.person.append(&mut back.person.clone());
    // }
    //let bytes = smith.rust2binary(&back, &typ).unwrap();
  
    for _ in 0..5{
        let start = Instant::now();
        back = smith.binary2rust(&bytes, &typ).unwrap();
        let duration = start.elapsed();
        println!("deserialisation: {duration:?} ({})",Byte::from_bytes(bytes.len() as u128).get_appropriate_unit(false).to_string());
    }

    for _ in 0..10{
        let start = Instant::now();
        _=smith.rust2binary(&back, &typ).unwrap();
        let duration = start.elapsed();
        println!("serialisation: {duration:?}");
    }

    
   // _=fs::write("./out.bin", &bytes);
}

pub mod Types {
    // smith_rsmacro::generate_bindings!("./schema.smith");
    #[doc = "this is a comment"]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Person<'a> {
    pub index: u64,
    pub alive: bool,
    pub isActive: bool,
    pub picture: &'a str,
    pub age: u8,
    pub name: String,
    pub latitude: f32,
    pub longitude: f32,
    pub friends: Vec<SimplePerson<'a>>,
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SimplePerson<'a> {
    pub id: u8,
    pub name: &'a str,
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Root<'a> {
    #[serde(borrow)]
    pub person: Vec<Person<'a>>,
}

}
