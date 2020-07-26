//python ex6
fn main() {
    let x = "There are 10 types of people.";
    let binary = "binary";
    let do_not = "don't";
    println!("{}", x);
    println!("Those who know {} and those who {}.", binary, do_not);

    println!("I said {}.", x);
    println!("I also said: 'Those who know {} and those who {}'.", binary, do_not);
    let hilarious: bool = false;
    let joke_evaluation = "Isn't that joke so funny?";
    println!("{} {}",joke_evaluation, hilarious);

    let w = String::from("This is the left side of...");
    let e = String::from("a string with a right side.");

    println!("{}", w + &e);
}
