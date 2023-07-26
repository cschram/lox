pub const EXPRESSION_TEST: &str = r#"
    "foo" + (1 + (3 / 2) - (8 * 4))
"#;

pub const VARIABLE_TEST: &str = r#"
    var i = 5;
    var foo = "bar";
    var is_okay = true;
"#;

pub const PRINT_TEST: &str = r#"
    var pi = 3.14;
    print pi;
    var foo;
    print foo;
"#;

pub const BLOCK_SCOPE_TEST: &str = r#"
    var foo = "foo";
    {
        print foo;
        var foo = "bar";
        print foo;
    }
"#;

pub const CONTROL_FLOW_TEST: &str = r#"
    if (true and (nil or "truthy")) {
        print "true";
    } else {
        print "false";
    }
    if (false) {
        print "false";
    } else {
        print "true";
    }
"#;

pub const WHILE_LOOP_TEST: &str = r#"
    var index = 4;
    while (index > 0) {
        print index;
        index = index - 1;
    }
"#;

pub const FOR_LOOP_TEST: &str = r#"
    var index = 42;
    for (var index = 0; index < 4; index = index + 1) {
        print index;
    }
    print index;
"#;

pub const BUILTINS_TEST: &str = r#"
    print time();
"#;

pub const FUNCTION_TEST: &str = r#"
    fun greet(name) {
        fun greeting() {
            return "Hello, " + name + "!";
        }
        print greeting();
    }
    fun get_name() {
        return "world";
    }
    greet(get_name());
"#;

pub const FUNCTION_CLOSURE_TEST: &str = r#"
    fun make_counter() {
        var i = 0;
        fun count() {
            i = i + 1;
            print i;
        }

        return count;
    }

    var counter = make_counter();
    counter();
    counter();
"#;

pub const SHADOWING_TEST: &str = r#"
    var a = "global";
    {
        fun print_a() {
            print a;
        }

        print_a();
        var a = "block";
        print_a();
    }
"#;

pub const CLASS_TEST: &str = r#"
    class Greeter {
        init(greeting) {
            this.greeting = greeting;
        }

        greet(name) {
            print this.greeting + ", " + name + "!";
        }

        make_greet(name) {
            fun greet() {
                print this.greeting + ", " + name + "!";
            }
            return greet;
        }
    }
    var greeter = Greeter("Hello");
    greeter.greet("world");
    var greet = greeter.make_greet("friends");
    greet();
"#;
// pub const CLASS_TEST: &str = r#"
//     class Greeter {
//         init(greeting) {
//             this.greeting = greeting;
//         }

//         greet(name) {
//             print this.greeting + ", " + name;
//         }
//     }

//     var greeter = Greeter("Hello");
//     greeter.greet("world");
// "#;

#[allow(dead_code)]
pub const CLASS_INHERITANCE_TEST: &str = r#"
    class Greeter {
        init(greeting) {
            this.greeting = greeting;
        }

        greet(name) {
            print this.greeting + ", " + name;
        }
    }

    class HelloGreeter < Greeter {
        init() {
            super.init("Hello");
        }
    }

    class HowdyGreeter < Greeter {
        init() {
            super.init("Howdy")
        }
    }

    var hello = HelloGreeter();
    var howdy = HowdyGreeter();
    hello.greet("world");
    howdy.greet("partner");
"#;
