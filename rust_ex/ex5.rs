//python ex5
fn main() {
    let my_name= String::from("Zed A.Shaw");
    let my_age: u32 = 35 ;     
    let my_height: u32 = 74 ;
    let my_weight: u32 = 180;  
    let my_eyes: String = String::from("Blue");
    let my_teeth = "White";
    let my_hair = "Brown";
    println!("Let's talk about {}", my_name);
    println!("He's {} inches tall.", my_height);
    println!("He's {} pounds heavy.", my_weight);
    println!("He's got {} eyes and {} hair." , my_eyes, my_hair);
    println!("Actually that's not too heavy");
    println!("His teeth are usually {} depending on the coffee." , my_teeth);

    println!("If I add {}, {} and {} I get {}." , my_age, my_height, my_weight, my_age + my_height + my_weight);

    let my_greeting = "Hello,   ";
    let my_first_name = "Joseph";
    let my_last_name = "Pan";
    let my_age = 24;

    println!("{} my name is {} {}, and I'm {} years old.", my_greeting, my_first_name, my_last_name, my_age);


    let inches_per_centimeters = 2.54;
    let pounds_per_kilo = 0.45359237;

    println!("He's {} centimeters tall.", my_height as f64*inches_per_centimeters);
    println!("He's {} kilos heavy.", my_weight as f64*pounds_per_kilo);
}
