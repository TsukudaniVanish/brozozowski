fn main() {
  use std::env;
  let args: Vec<_> = env::args().collect();
  if args.len() < 3 {
    panic!("check [divider] [regular_expr]")
  }

  let mut exp = match parse_exp(args[2].chars().collect::<Vec<char>>()) {
      Ok((r, res)) => if res.len() == 0 {
        r 
      } else {
        panic!("can't parse regular expression")
      }
      Err(s) => panic!("{}", s)
  };

  for c in args[1].chars() {
    exp = match divide_exp(&c, exp) {
        Ok(r) => match *r {
            Exp::Empty => {
                println!("not match");
                return 
            }
            s => Box::new(s)
        }
       Err(s) => panic!("{}", s)
    };
  };

  if has_empty_word(&exp) {
    println!("match!")
  } else {
    println!("not match")
  }
    
}

/*bnf
    exp = or_exp
    or_exp = con_exp ("|" con_exp)*
    con_exp = star_exp*
    star_exp = primary "*"
    primary = __alphabet_or_number__ | "(" exp ")"
 */

#[derive(Clone, Debug)]
enum Exp {
    Concat(Vec<Box<Exp>>),
    Or(Vec<Box<Exp>>),
    Star(Box<Exp>),
    Single(char),
    Zero,
    Empty,
}

impl Exp {
    pub fn new_single(c: char) -> Box<Exp> {
        Box::new(Exp::Single(c))
    }

    pub fn new_zero() -> Box<Exp> {
        Box::new(Exp::Zero)
    }
} 

fn or(left: Box<Exp>, right: Box<Exp>) -> Box<Exp> {
    match *left {
        Exp::Empty => right,
        Exp::Or(v) => if v.len() > 0{
            match *right {
                Exp::Empty => Box::new(Exp::Or(v)),
                Exp::Or(w) => {
                    let mut exps = v.clone();
                    exps.append(&mut w.clone());
                    Box::new(Exp::Or(exps))
                }
                _ => {
                    let mut exps = v.clone();
                    exps.push(right);
                    Box::new(Exp::Or(exps))
                }
            }
        } else {
            right
        }
        _ => match *right {
            Exp::Empty => left,
            Exp::Or(v) => {
                let mut exps = vec![left];
                exps.append(&mut v.clone());
                Box::new(Exp::Or(exps))
            }
            _ => Box::new(Exp::Or([left, right].to_vec()))
        }
    }
}

fn concat(left: Box<Exp>, right: Box<Exp>) -> Box<Exp> {
    match *left {
        Exp::Empty => Box::new(Exp::Empty),
        Exp::Zero => right,
        Exp::Concat(v) => if v.len() == 0 {
            right 
        } else {
            match *right {
                Exp::Empty => Box::new(Exp::Empty),
                Exp::Zero => Box::new(Exp::Concat(v)),
                Exp::Concat(w) => {
                    let mut exps = v.clone();
                    exps.append(&mut w.clone());
                    Box::new(Exp::Concat(exps))
                }
                _ => {
                    let mut exps = v.clone();
                    exps.push(right);
                    Box::new(Exp::Concat(exps))
                }
            }
        }
        _ => {
            match *right {
                Exp::Empty => Box::new(Exp::Empty),
                Exp::Zero => left,
                Exp::Concat(v) => {
                    let mut exps = vec![left];
                    exps.append(&mut v.clone());
                    Box::new(Exp::Concat(exps))
                }
                _ => Box::new(Exp::Concat([left, right].to_vec()))
            }
        }
    }
}



fn has_empty_word(boxed_exp:& Box<Exp>) -> bool {
    match &**boxed_exp {
        Exp::Concat(v) => {
            if v.len() == 0 {
                false 
            } else {
                v.iter().map(|exp| has_empty_word(exp)).all(|b| b)
            }
        }
        Exp::Or(v) => {
            if v.len() == 0 {
                false 
            } else {
                v.iter().map(|exp| has_empty_word(exp)).any(|b| b)
            }
        }
        Exp::Star(_) => true,
        Exp::Single(_) => false,
        Exp::Zero => true,
        Exp::Empty => false,
    }
}

#[test]
fn has_empty_word_test() {
    let t1 = Box::new(Exp::Or([Box::new(Exp::Star(Exp::new_single('a'))), Exp::new_single('b')].to_vec()));
    assert!(has_empty_word(&t1), "a*|b has empty word");
}

fn divide_exp(a :&char, exp : Box<Exp>) -> Result<Box<Exp>, String> {
    return divide_or(a, exp)
}

fn divide_or(a: &char, exp: Box<Exp>) -> Result<Box<Exp>, String> {
    match *exp {
        Exp::Or(v) => {
            let mut exps: Vec<Box<Exp>> = Vec::new();
            for e in v.iter() {
                match divide_concat(a, e.to_owned()) {
                    Ok(r) => exps.push(r),
                    Err(s) => return Err(s)
                }
            }
            Ok(Box::new(Exp::Or(exps)))
        }
        _ => divide_concat(a, exp)
    }
}

fn divide_concat(a :&char, exp: Box<Exp>) -> Result<Box<Exp>, String> {
    match *exp {
        Exp::Concat(v) => {
            if v.len() == 0 {
                Ok(Box::new(Exp::Empty))
            } else {
                let tail = v[1..].to_vec();
                match divide_star(a, v[0].clone()) {
                    Ok(r) => {
                        if has_empty_word(&v[0]) {
                            match divide_concat(a, Box::new(Exp::Concat(tail))) {
                                Ok(r2) => {
                                    Ok(or(r, r2))
                                },
                                Err(s) => Err(s)
                            }
                        } else {
                            Ok(r)
                        }
                    }
                    Err(s) => Err(s),
                }
            }
        }
        _ => divide_star(a, exp)
    }
}

fn divide_star(a :&char, exp: Box<Exp>) -> Result<Box<Exp>, String> {
    match *exp {
        Exp::Star(s) => match divide_primary(a, s.clone()) {
            Ok(r)  => Ok(concat(r, Box::new(Exp::Star(s.clone())))),
            Err(s) => Err(s)
        }
        s => divide_primary(a, Box::new(s))
    }
}

fn divide_primary(a: &char, exp: Box<Exp>) -> Result<Box<Exp>, String> {
    match *exp {
        Exp::Concat(_) => divide_exp(a, exp),
        Exp::Or(_) => divide_exp(a, exp),
        Exp::Star(_) => divide_exp(a, exp),
        Exp::Single(c) => {
            if *a == c {
                Ok(Box::new(Exp::Zero))
            } else {
                Ok(Box::new(Exp::Empty))
            }
        }
        Exp::Empty => Ok(Box::new(Exp::Zero)),
        Exp::Zero => Ok(Box::new(Exp::Zero))
    }
}

fn parse_exp(input :Vec<char>) -> Result<(Box<Exp>, Vec<char>), String> {
    parse_or(input, Box::new(Exp::Empty))
}

fn parse_or(input: Vec<char>, exp: Box<Exp>) -> Result<(Box<Exp>, Vec<char>), String> {
    match parse_concat(input, Box::new(Exp::Zero)) {
        Ok((r, res)) => if res.len() == 0 {
            Ok((or(exp, r), res))
        } else {
            if res[0] == '|' {
                parse_or(res[1..].to_vec(), or(exp, r))
            } else {
                Err(String::from("|: not fond"))
            }
        }
        Err(s) => Err(s)
    }
}

fn parse_concat(input: Vec<char>, exp: Box<Exp>) -> Result<(Box<Exp>, Vec<char>), String> {
    match parse_star(input) {
        Ok((r, res)) => if res.len() == 0 {
            Ok((concat(exp, r), res))
        } else if res[0] == '|' || res[0] == ')' {
            Ok((concat(exp, r), res))
        } else {
            parse_concat(res[1..].to_vec(), concat(exp, r))
        }
        Err(s) => Err(s)
    }
}

fn parse_star(input:Vec<char>) -> Result<(Box<Exp>, Vec<char>), String> {
    match parse_primary(input) {
        Ok((r, res)) => if res.len() > 0 && res[0] == '*' {
            Ok((Box::new(Exp::Star(r)), res[1..].to_vec()))
        } else { Ok((r, res)) }
        Err(s) => Err(s)
    }
}

fn parse_primary(input: Vec<char>) -> Result<(Box<Exp>, Vec<char>), String> {
    if input.len() == 0 {
        Ok((Box::new(Exp::Zero), input))
    } else { 
        match input[0] {
            '(' => {
                match parse_exp(input[1..].to_vec()) {
                    Ok((r, res)) => if res[0] == ')' {
                        Ok((r, res[1..].to_vec()))
                    } else {
                        Err(String::from("Can't find )"))
                    }
                    Err(s) => Err(s)
                }
            }
            c => Ok((Box::new(Exp::Single(c)), input[1..].to_vec())),
        }
    }
}

#[test]
fn parse_exp_test() {
    let v = "a|b".chars().collect::<Vec<char>>();
    match parse_exp(v) {
        Ok((r, res)) => {
            assert!(res.len() == 0, "res is not empty: \n res: {:#?}", res);
            match *r {
                Exp::Or(v) => {
                    assert!(v.len() == 2, "not correct or: {:#?}", v);
                    let v0 = &v[0];
                    let v1 = &v[1];
                    match &**v0 {
                        Exp::Single(c) => assert!(*c == 'a', "expected a but got {}", c),
                        e => panic!("{:#?} is not expected", e),
                    }
                    match &**v1 {
                        Exp::Single(c) => assert!(*c == 'b', "expected b but got {}", c),
                        e => panic!("{:#?} is not expected", e)
                    }
                }
                e => panic!("{:#?} is not expected", e)
            }
        }
        Err(s) => panic!("{}", s),
    }
}