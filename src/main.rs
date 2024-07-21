use reactivity::{computed, effect, use_ref};

#[derive(Clone, Copy)]
struct A {
    value: i32,
}

fn main() {
    let a0 = use_ref(0);
    let a1 = use_ref(1);
    let s0 = use_ref(A { value: 2 });

    effect({
        let a0 = a0.clone();
        move || {
            println!("A0 is: {}", a0.get());
        }
    });

    let a2 = computed({
        let a0 = a0.clone();
        let a1 = a1.clone();
        move || a0.get() + a1.get()
    });

    // let a2 = computed(move || *a0 + *a1);

    effect(move || {
        println!("A2 is: {}", a2.get());
    });

    a0.set(2);

    effect({
        let s0 = s0.clone();
        move || {
            println!("S0 is: {}", s0.get().value);
        }
    });

    s0.set(A { value: 3 });
    s0.update(|s| A { value: s.value + 1 });
}
