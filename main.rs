use std::{fmt::Display, ops::Deref, rc::Rc};
/// 捕获
/// 宏模式中还可以包含捕获。这允许输入匹配在某种通用语法基础上进行，并使得结果被捕获进某个变量中。此变量可在输出中被替换使用。
/// 捕获由$符号紧跟一个标识符(identifier)紧跟一个冒号(:)紧跟捕获种类组成。捕获种类须是如下之一：
/// item: 条目，比如函数、结构体、模组等。
/// block: 区块(即由花括号包起的一些语句加上/或是一项表达式)。
/// stmt: 语句
/// pat: 模式
/// expr: 表达式
/// ty: 类型
/// ident: 标识符
/// path: 路径 (例如 foo, ::std::mem::replace, transmute::<_, int>, …)
/// meta: 元条目，即被包含在 #[...]及#![...]属性内的东西。
/// tt: 标记树

///模式中可以包含重复。这使得匹配标记序列成为可能。重复的一般形式为$ ( ... ) sep rep
/// $ 是字面标记。
/// ( ... ) 代表了将要被重复匹配的模式，由小括号包围。
/// sep是一个可选的分隔标记。常用例子包括,和;。
/// rep是重复控制标记。当前有两种选择，分别是* (代表接受0或多次重复)以及+ (代表1或多次重复)。目前没有办法指定“0或1”或者任何其它更加具体的重复计数或区间。

macro_rules! name {
    () => {
        1 + 3
    };
    ($e:expr) => {
        $e * 3
    };
    ($a:expr,$b:expr,c:expr) => {
        $a * ($b + $c)
    };
    ($($e:expr) ; *) => {{
        let mut v = Vec::new();

        $(
        v.push(format!("{}",$e));
        )*

        v

    }};

    ($i:ident,$item:item)=>{
        $item

        fn $i(){

        }
    };

    ($item:item)=>{
        $item
    };

    ($i:ident +)=>{};


}

macro_rules! using_a {
    ($i:ident,$e:expr) => {{
        let $i = 42;
        $e
    }};
}

// name! {struct Test {
//     name: usize,
// }}
//
// name! {ab,
//     fn ww() {
//         println!("www")
//     }
// }

macro_rules! what_is {
    (self) => {
        "the keyword `self`"
    };
    ($i:ident) => {
        concat!("the identifier `", stringify!($i), "`")
    };
}

macro_rules! call_with_ident {
    ($c:ident($i:ident)) => {
        $c!($i)
    };
}

macro_rules! each_tt {
    () => {};
    ($_tt:tt $($rest:tt)*) => {
        each_tt!($($rest)*);
    };
}
fn main() {
    // let x = name! {2 ; 2 ; 2};
    //
    // println!("{:?}", x);
    // ww();
    //
    // name!(x+);

    /// https://www.bookstack.cn/read/DaseinPhaos-tlborm-chinese/mbe-min-hygiene.md
    // let four = using_a!(a, a / 10);
    // println!("{}", what_is!(self));
    //
    // println!("{}", call_with_ident!(what_is(self)))
    // each_tt!(foo bar baz quux);
    //
    // each_tt!(spim wak plee whum);
    //
    // each_tt!(trom qlip winp xod);
    let string1 = String::from("long string is long");

    let result;

    {
        let string2 = String::from("xyz");

        result = longest(string1.as_str(), string2.as_str());
    }
    println!("The longest string is {}", result);
}

struct W<'a> {
    v: &'a str,
}

struct A {
    b: i32,
}

impl<'a> W<'a> {
    fn a<'b>(&self, t: &'b str) -> &'a str
    where
        'a: 'b,
    {
        self.v
    }
}

fn longest<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    // if x.len() > y.len() {
    //     x
    // } else {
    //     y
    // }
    x
}

fn longest_w<'a, T>(x: &'a str, y: &'a str, ann: T) -> &'a str
where
    T: Display,
{
    println!("{}", ann);

    if x.len() > y.len() {
        x
    } else {
        y
    }
}

#[derive(Debug)]
struct Foo;

impl Foo {
    fn mutate_and_share(&mut self) -> &Self {
        &*self
    }
    fn share(&self) {}
}

// #[test]
// fn test1() {
//     let mut foo = Foo;
//     let loan = foo.mutate_and_share();
//     foo.share();
//
//     println!("{:?}", loan);
// }

// #[test]
// fn test2() {
//     use std::collections::HashMap;
//     use std::hash::Hash;
//
//     fn get_default<'a, K, V>(map: &'a mut HashMap<K, V>, k: K) -> &'a mut V
//     where
//         K: Clone + Eq + Hash,
//         V: Default,
//     {
//         if let Some(value) = map.get_mut(&k) {
//             return value;
//         }
//
//         map.insert(k.clone(), V::default());
//         map.get_mut(&k).unwrap()
//     }
// }

#[test]
fn test3() {
    fn f<'a, T>(x: *const T) -> &'a T {
        unsafe { &*x }
    }

    // 表示类型T活得比'a 久
    struct Ref<'a, T: 'a> {
        r: &'a T,
    }
}

#[test]
fn test4() {
    // let s1: Box<str> = Box::new("Hello there!" as str);
    let s1: Box<str> = "hello".into();
}

#[test]
fn test5() {
    let s = gen_static_str();
    println!("{}", s);
}

fn gen_static_str() -> &'static str {
    let mut s = String::new();
    s.push_str("hello");
    Box::leak(s.into_boxed_str())
}

#[derive(Debug)]
struct Person {
    name: String,
    age: u8,
}

impl Person {
    fn new(name: String, age: u8) -> Self {
        Self { name, age }
    }

    fn display(self: &mut Person, age: u8) {
        let Person { name, age } = &self;
    }
}

#[test]
fn test6() {
    let x = 5;
    let y = &x;

    assert_eq!(5, x);
    assert_eq!(5, *y);

    let x = Box::new(1);

    let sum = *x + 1;
}

struct MyBox<T>(T);

impl<T> MyBox<T> {
    fn new(x: T) -> MyBox<T> {
        MyBox(x)
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[test]
fn test7() {
    let y = MyBox::new(3);

    assert_eq!(3, *y);

    let s = MyBox::new(String::from(""));
    display(&s);
}

fn display(s: &str) {}
fn ww<T: ?Sized + Send + Display + Sync + 'static>(t: &T) {
    println!("{}", t);
}

#[test]
fn test8() {
    struct HasDrop1;
    struct HasDrop2;

    impl Drop for HasDrop1 {
        fn drop(&mut self) {
            println!("Dropping HasDrop1!");
        }
    }
    impl Drop for HasDrop2 {
        fn drop(&mut self) {
            println!("Dropping HasDrop2!");
        }
    }
    struct HasTwoDrops {
        one: HasDrop1,
        two: HasDrop2,
    };

    impl Drop for HasTwoDrops {
        fn drop(&mut self) {
            println!("Dropping HasTwoDrops!");
        }
    }

    struct FOO;

    impl Drop for FOO {
        fn drop(&mut self) {
            println!("Dropping FOO!");
        }
    }

    let _x = HasTwoDrops {
        one: HasDrop1,
        two: HasDrop2,
    };

    let _foo = FOO;

    println!("Running");
}

#[test]
fn test9() {
    // #[derive(Copy)]
    struct Foo;

    impl Drop for Foo {
        fn drop(&mut self) {}
    }
}

#[test]
fn test10() {
    struct Owner {
        name: String,
    }

    struct Gadget {
        id: i32,
        owner: Rc<Owner>,
    }

    let gadget_owner = Rc::new(Owner {
        name: "wwdfds".to_string(),
    });

    println!("strong_count = {}", Rc::strong_count(&gadget_owner));
    let gadget1 = Gadget {
        id: 1,
        owner: Rc::clone(&gadget_owner),
    };
    let gadget2 = Gadget {
        id: 2,
        owner: Rc::clone(&gadget_owner),
    };

    drop(gadget_owner);
    drop(gadget1);

    println!("strong_count = {}", Rc::strong_count(&gadget2.owner));

    println!("gadget2 = {}", (*gadget2.owner).name);
}

#[test]
fn test22() {
    struct Cacher<T, E>
    where
        T: Fn(E) -> E,
        E: Copy,
    {
        query: T,
        value: Option<E>,
    }

    impl<T, E> Cacher<T, E>
    where
        T: Fn(E) -> E,
        E: Copy,
    {
        fn new(query: T) -> Cacher<T, E> {
            Cacher { query, value: None }
        }

        fn value(&mut self, arg: E) -> E {
            match self.value {
                Some(v) => v,
                None => {
                    let v = (self.query)(arg);
                    self.value = Some(v);
                    v
                }
            }
        }
    }

    #[test]
    fn call_with_different_values() {
        let mut c = Cacher::new(|a| a);

        let v1 = c.value(1);
        let v2 = c.value(2);

        assert_eq!(v2, 1);
    }
}

#[test]
fn test81() {
    fn invalid_output<'a>() -> &'a str {
        "foo"
    }

    fn invalid_output1() -> &'static str {
        "foo"
    }

    fn invalid_output2() -> String {
        "foo".to_string()
    }

    /* 让下面的代码工作 */
    fn failed_borrow<'a, 'b: 'a>() {
        // b的生命周期大于a
        let _x: &'b i32 = &12;

        // ERROR: `_x` 活得不够久does not live long enough
        let y: &'a i32 = &_x;

        // 在函数内使用 `'a` 将会报错，原因是 `&_x` 的生命周期显然比 `'a` 要小
        // 你不能将一个小的生命周期强转成大的
    }

    fn print_refs<'a, 'b>(x: &'a i32, y: &'b i32) {
        println!("x is {} and y is {}", x, y);
    }
    let b = Box::new(1);
    let a = Box::leak(b);
}
