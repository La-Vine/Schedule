fn main()
{
    let mut  i = 0;
    
    let args = std::env::args();
    for arg in args {
        println!("arg {}: {}\n", i, arg);
        i+=1;
    }
    let states:Vec<&str> = vec![
        "California", "Oregon",
        "Washington", "Texas"
    ];

    let  num_states = 5;

    
    for i in 0..num_states as usize{
        println!("state {}: {}", i, states[i]);
    }

}