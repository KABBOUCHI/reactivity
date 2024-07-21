use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Ref<T> {
    value: Arc<Mutex<T>>,
    effects: Arc<Mutex<Vec<usize>>>,
}

impl<T: Copy + 'static> Ref<T> {
    pub fn new(value: T) -> Self {
        Ref {
            value: Arc::new(Mutex::new(value)),
            effects: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get(&self) -> T {
        if let Some(current_effect) = CURRENT_EFFECT.with(|ce| *ce.lock().unwrap()) {
            self.effects.lock().unwrap().push(current_effect);
        }
        *self.value.lock().unwrap()
    }

    pub fn set(&self, new_value: T) {
        *self.value.lock().unwrap() = new_value;
        self.trigger();
    }

    fn trigger(&self) {
        for &effect_id in self.effects.lock().unwrap().iter() {
            if let Some(effect) =
                EFFECTS.with(|effects| effects.lock().unwrap().get(&effect_id).cloned())
            {
                let mut effect = effect.lock().unwrap();
                effect();
            }
        }
    }

    pub fn update<F>(&self, update_fn: F)
    where
        F: FnOnce(T) -> T,
    {
        let value = update_fn(self.get());
        self.set(value);
    }
}

pub struct Computed<T> {
    value: Arc<Mutex<T>>,
    compute_fn: Arc<dyn Fn() -> T>,
    effects: Arc<Mutex<Vec<usize>>>,
}

impl<T: Copy + 'static> Computed<T> {
    pub fn new<F>(compute_fn: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        let computed = Computed {
            value: Arc::new(Mutex::new(compute_fn())),
            compute_fn: Arc::new(compute_fn),
            effects: Arc::new(Mutex::new(Vec::new())),
        };

        let computed_rc = Arc::new(computed);
        let computed_clone = Arc::clone(&computed_rc);
        effect(move || {
            let new_value = (computed_clone.compute_fn)();
            *computed_clone.value.lock().unwrap() = new_value;
            computed_clone.trigger();
        });

        (*computed_rc).clone()
    }

    pub fn get(&self) -> T {
        if let Some(current_effect) = CURRENT_EFFECT.with(|ce| *ce.lock().unwrap()) {
            self.effects.lock().unwrap().push(current_effect);
        }
        *self.value.lock().unwrap()
    }

    fn trigger(&self) {
        for &effect_id in self.effects.lock().unwrap().iter() {
            if let Some(effect) =
                EFFECTS.with(|effects| effects.lock().unwrap().get(&effect_id).cloned())
            {
                let mut effect = effect.lock().unwrap();
                effect();
            }
        }
    }
}

impl<T> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Computed {
            value: Arc::clone(&self.value),
            compute_fn: Arc::clone(&self.compute_fn),
            effects: Arc::clone(&self.effects),
        }
    }
}

impl<T> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Ref {
            value: Arc::clone(&self.value),
            effects: Arc::clone(&self.effects),
        }
    }
}

thread_local! {
    static CURRENT_EFFECT: Mutex<Option<usize>> = Mutex::new(None);
    static EFFECTS: Mutex<HashMap<usize, Arc<Mutex<dyn FnMut()>>>> = Mutex::new(HashMap::new());
}

pub fn effect<F>(f: F)
where
    F: FnMut() + 'static,
{
    let effect_id = EFFECTS.with(|effects| {
        let mut effects = effects.lock().unwrap();
        let id = effects.len();
        effects.insert(id, Arc::new(Mutex::new(f)));
        id
    });

    let effect_ref = Arc::new(move || {
        CURRENT_EFFECT.with(|ce| {
            *ce.lock().unwrap() = Some(effect_id);
        });

        EFFECTS.with(|effects| {
            if let Some(effect) = effects.lock().unwrap().get(&effect_id).cloned() {
                let mut effect = effect.lock().unwrap();
                effect();
            }
        });

        CURRENT_EFFECT.with(|ce| {
            *ce.lock().unwrap() = None;
        });
    });

    effect_ref();
}

pub fn computed<F, T>(compute_fn: F) -> Computed<T>
where
    F: Fn() -> T + 'static,
    T: Copy + 'static,
{
    Computed::new(compute_fn)
}

pub fn use_ref<T: Copy + 'static>(value: T) -> Ref<T> {
    Ref::new(value)
}
