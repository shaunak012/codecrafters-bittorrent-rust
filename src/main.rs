use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode

fn bencode_ending_index(encoded_value: &str) -> usize {
    if encoded_value.chars().next().unwrap().is_digit(10) {
        let colon_index = encoded_value.find(':').unwrap();
        let number_string = &encoded_value[..colon_index];
        let number = number_string.parse::<usize>().unwrap();
        return colon_index + 1 + number ;
    } else if encoded_value.starts_with("i"){
        let ending_index = encoded_value.find('e').expect("Invalid bencoded integer format");
        return ending_index+1 ;
    } else if encoded_value.starts_with("l") || encoded_value.starts_with("d"){
        let mut counter = 0;
        let mut i = 0;
        let chars: Vec<char> = encoded_value.chars().collect();
        while i < chars.len(){
            match chars[i] {
                'l' => counter+= 1,
                'd' => counter+= 1,
                'i' => {
                    // println!("Entry at {}",i);
                    i+=1;
                    while chars[i].is_digit(10){
                        i+=1;
                    }
                    // println!("Exit at {}",i);
                },
                'e' => {
                    // println!("Entry at {}, {}",i, &counter);
                    counter-=1;
                    if counter ==0{
                        break;
                    }
                    // println!("Exit at {}, {}",i, &counter);
                },
                _ =>{
                    // println!("Entry at {}",i);
                    if chars[i].is_digit(10) {
                        let mut j = i;
                        while chars[j] != ':' { j += 1; }
                        let len: usize = encoded_value[i..j].parse().unwrap();
                        i = j + len; 
                    }
                    // println!("Exit at {}",i);
                }
            }
            i+=1;
        }
        return i+1;
    } else{
        panic!("Invalid string : {}",encoded_value);
    }
}

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    // If encoded_value starts with a digit, it's a number
    
    let ending_index = bencode_ending_index(encoded_value);
    // println!("Ending Index: {}",ending_index);
    if encoded_value.chars().next().unwrap().is_digit(10) {
        // Example: "5:hello" -> "hello"
        let colon_index = encoded_value.find(':').unwrap();
        let string = &encoded_value[colon_index + 1..ending_index];
        return serde_json::Value::String(string.to_string());
    } else if encoded_value.starts_with("i"){
        let number_part = &encoded_value[1..ending_index-1];
        let number=number_part.parse::<i64>().unwrap();
        return serde_json::Value::Number(number.into());
    } else if encoded_value.starts_with("l"){
        let mut list = vec![];
        let mut current_index = 1; 
        while current_index < ending_index-1{
            // println!("List left: {}",&encoded_value[current_index..]);
            let element_end= bencode_ending_index(&encoded_value[current_index..]);
            // println!("Element End: {}",current_index+element_end);
            list.push(decode_bencoded_value(&encoded_value[current_index..]));
            current_index+=element_end;
        }
        return serde_json::Value::Array(list);
    } else if encoded_value.starts_with("d"){
        let mut list = serde_json::Map::new();
        let mut current_index =1;
        while current_index < ending_index-1{
            let key_end= bencode_ending_index(&encoded_value[current_index..]);
            let key = match decode_bencoded_value(&encoded_value[current_index..]){
                    serde_json::Value::String(k) => k,
                    k => {
                        panic!("dict keys must be strings, not {k:?}");
                    }
                };
            current_index+=key_end;
            let value_end = bencode_ending_index(&encoded_value[current_index..]);
            let value = decode_bencoded_value(&encoded_value[current_index..]);
            current_index+=value_end;
            list.insert(key,value);
        }
        return serde_json::Value::Object(list)
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}

// Usage: your_program.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        eprintln!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
