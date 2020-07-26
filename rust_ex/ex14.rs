fn print_arguments(argv:&mut std::env::Args)
{
    for arg in argv {
        print_letters(arg);
    }   
}

fn print_letters(arg:String)
{
    let mut argn = arg.chars();

    while let Some(i) = argn.next(){
        if (can_print_it(i) == true) {
            println!("'{}' == {} ", i, i as u8);
        }
    }
}

fn can_print_it(ch:char)->bool
{
    return ch.is_alphabetic() ||  ch.is_whitespace();
}

fn main()
{
    let mut args = std::env::args();
    print_arguments( & mut args);
    
}