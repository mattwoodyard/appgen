

#[derive(Debug, Eq, PartialEq)]
pub enum Expr {
    EPrim(String),
    EPair(String, Box<Expr>),
    EList(Vec<Box<Expr>>),
    EConstruct(String, String, Vec<Box<Expr>>)
}

impl Expr {
    fn pair(a: &str, b: &str) -> Expr {
        Expr::EPair(a.into(), Box::new(Expr::EPrim(b.into())))
    }
}

named!(econstruct<CompleteStr, Expr>,
    do_parse!(
        k: ws!(some_words) >>
        ws!(tag!("=")) >>
        t: ws!(some_words) >>
        ws!(tag!("(")) >>
        b: ws!(elist_flat) >>
        ws!(tag!(")")) >>
        (Expr::EConstruct(k.into(), t.into(), b))));

named!(eprim<CompleteStr, Expr>,
    map!(some_words, |x| Expr::EPrim(x.into())));

named!(epair<CompleteStr, Expr>,
    map!(separated_pair!(some_words, ws!(tag!("=")), expr),
    |(k, v)| {Expr::EPair(k.into(), Box::new(v))}));

named!(elist<CompleteStr, Expr>,
    map!(elist_flat, |v| Expr::EList(v)));


named!(elist_flat<CompleteStr, Vec<Box<Expr>>>,
    map!(separated_nonempty_list!(ws!(tag!(",")), expr), |v| v.into_iter().map(Box::new).collect()));

named!(paren_list<CompleteStr, Expr>,
    do_parse!(
        tag!("(") >>
        e: elist >>
        tag!(")") >> 
        (e)));

named!(expr<CompleteStr, Expr>, alt!(econstruct| paren_list | epair | eprim));
named!(top_level_expr<CompleteStr, Expr>, alt!(ws!(econstruct)|ws!(elist)|ws!(epair)|ws!(eprim)));
named!(top_level_def<CompleteStr, Vec<Expr>>, many1!(ws!(top_level_expr)));

pub fn basic_word<T>(input: T) -> IResult<T, T, u32>
where
    T: Slice<Range<usize>> + Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
    T: InputIter + InputLength + AtEof,
    <T as InputIter>::RawItem: AsChar,
{
    match input.position(|item| {
            let c = item.as_char().clone();
            non_terminal(c)
        })
        {
        Some(n) => Ok((input.slice(n..), input.slice(..n))),
        None => {
            Ok((input.slice(input.input_len()..), input))
        }
    }
}

fn non_terminal(c: char) -> bool {
    c != ',' && c != '\\' && c != ')' && c != '(' && c != '='
}

/* named!(some_words<CompleteStr, String>, */
/*     map!(escaped!(basic_word, '\\',  one_of!("(),=")), |s| String::from(s.as_ref()))); */



// Ugggh thats filthy
named!(some_words<CompleteStr, String>, map!(
    separated_nonempty_list!(space1, alphanumeric1), |v| v.into_iter().map(|i| String::from(i.as_ref())).collect::<Vec<String>>().join(" ") ));


#[test]
fn test_simple() {
    println!("{:?}", econstruct(CompleteStr("x = type (foo)")));
    println!("{:?}", econstruct(CompleteStr("x = type (foo, bar = y, junk = (baz = a))")));
    println!("{:?}", top_level_def(CompleteStr("x = type (foo, bar = y, junk = (baz = a))")));
    println!("{:?}", top_level_def(CompleteStr("x = type (blah = foo, bar = y, junk = (baz = a))")));



    println!("{:?}", top_level_def(CompleteStr("transaction = table ( txntimestamp = field ( type = timestamp without time zone), id = field ( type = big serial ), merchant = field (type = varchar(256)))"))); 

    assert_eq!(epair(CompleteStr("x = y")).unwrap().1, Expr::EPair("x".into(), Box::new(Expr::EPrim("y".into()))));

    assert_eq!(elist(CompleteStr("x = y, a = b, c = d")).unwrap().1,
               Expr::EList(vec!(Box::new(Expr::pair("x", "y")),
                                Box::new(Expr::pair("a", "b")),
                                Box::new(Expr::pair("c", "d")))));
}


