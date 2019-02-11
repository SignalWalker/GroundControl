pub extern crate architect;
pub extern crate winit;

use architect::birch::Tree;
use architect::*;
use std::collections::HashMap;
use std::collections::HashSet;
use winit::*;

/// Eq, Hash, and PartialEq not implemented by winit::KeyboardInput, so...
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct KeyState {
    scancode: Option<ScanCode>,
    state: Option<ElementState>,
    shift: Option<bool>,
    ctrl: Option<bool>,
    alt: Option<bool>,
    logo: Option<bool>,
    virtual_keycode: Option<VirtualKeyCode>,
}

impl KeyState {
    fn subsumes(&self, other: &KeyState) -> bool {
        (self.scancode.is_none() || self.scancode == other.scancode)
            && (self.state.is_none() || self.state == other.state)
            && (self.virtual_keycode.is_none() || self.virtual_keycode == other.virtual_keycode)
            && (self.shift.is_none() || self.shift == other.shift)
            && (self.ctrl.is_none() || self.ctrl == other.ctrl)
            && (self.alt.is_none() || self.alt == other.alt)
            && (self.logo.is_none() || self.logo == other.logo)
    }

    fn subsumes_winit(&self, other: &KeyboardInput) -> bool {
        (self.scancode.is_none() || self.scancode.unwrap() == other.scancode)
            && (self.state.is_none() || self.state.unwrap() == other.state)
            && (self.virtual_keycode.is_none() || self.virtual_keycode == other.virtual_keycode)
            && (self.shift.is_none() || self.shift.unwrap() == other.modifiers.shift)
            && (self.ctrl.is_none() || self.ctrl.unwrap() == other.modifiers.ctrl)
            && (self.alt.is_none() || self.alt.unwrap() == other.modifiers.alt)
            && (self.logo.is_none() || self.logo.unwrap() == other.modifiers.logo)
    }
}

impl From<KeyboardInput> for KeyState {
    fn from(key: KeyboardInput) -> Self {
        KeyState {
            scancode: Some(key.scancode),
            state: Some(key.state),
            shift: Some(key.modifiers.shift),
            ctrl: Some(key.modifiers.ctrl),
            alt: Some(key.modifiers.alt),
            logo: Some(key.modifiers.logo),
            virtual_keycode: key.virtual_keycode,
        }
    }
}

impl PartialEq<KeyboardInput> for KeyState {
    fn eq(&self, other: &KeyboardInput) -> bool {
        self.subsumes_winit(other)
    }
}

impl Default for KeyState {
    fn default() -> KeyState {
        KeyState {
            scancode: None,
            state: None,
            shift: None,
            ctrl: None,
            alt: None,
            logo: None,
            virtual_keycode: None,
        }
    }
}

#[derive(Debug)]
pub struct KeyMap(KeyState, HashSet<String>);

impl KeyMap {
    fn subsumes(&self, key: &KeyState) -> bool {
        self.0.subsumes(key)
    }
}

impl PartialEq<KeyState> for KeyMap {
    fn eq(&self, key: &KeyState) -> bool {
        self.0 == *key
    }
}

impl PartialEq for KeyMap {
    fn eq(&self, map: &KeyMap) -> bool {
        self.0 == map.0
    }
}

impl Default for KeyMap {
    fn default() -> KeyMap {
        KeyMap(KeyState::default(), HashSet::new())
    }
}

pub struct ControlMason {
    pub key_tree: Tree<KeyMap>,
    pub active: HashSet<String>,
}

impl Default for ControlMason {
    fn default() -> Self {
        ControlMason {
            key_tree: Tree::with_root(KeyMap::default()),
            active: HashSet::new(),
        }
    }
}

impl ControlMason {
    pub fn handle_key(&mut self, mut key: KeyboardInput) -> Vec<&String> {
        let mut res = Vec::new();
        for i in self.key_tree.all_nearest(&key.into(), |a, b| a.subsumes(b)) {
            for s in &self.key_tree[i].value.1 {
                res.push(s);
                if key.state == ElementState::Pressed {
                    //println!("{} added to active.", s);
                    self.active.insert(s.clone());
                }
            }
        }
        if key.state == ElementState::Released {
            key.state = ElementState::Pressed;
            for i in self.key_tree.all_nearest(&key.into(), |a, b| a.subsumes(b)) {
                for s in &self.key_tree[i].value.1 {
                    //println!("{} removed from active.", s);
                    self.active.remove(s);
                }
            }
        }
        res
    }

    pub fn read_key(&mut self, arch: &Architect, key: &Element, action: String) {
        self.insert(
            {
                let mut scancode = None;
                let mut state = Some(ElementState::Pressed);
                let mut virtual_keycode = None;
                let mut shift = Some(false);
                let mut ctrl = Some(false);
                let mut alt = Some(false);
                let mut logo = Some(false);
                for (k, v) in &key.attr {
                    match k.as_str() {
                        "state" => state = ControlMason::attr_to_state(&v, &arch.stones),
                        "scan" => scancode = ControlMason::attr_to_scan(&v, &arch.stones),
                        "virtual" => {
                            virtual_keycode = ControlMason::attr_to_virtual(&v, &arch.stones)
                        }
                        "shift" => shift = ControlMason::attr_to_mod(&v, &arch.stones),
                        "ctrl" => ctrl = ControlMason::attr_to_mod(&v, &arch.stones),
                        "alt" => alt = ControlMason::attr_to_mod(&v, &arch.stones),
                        "logo" => logo = ControlMason::attr_to_mod(&v, &arch.stones),
                        _ => (),
                    }
                }
                KeyState {
                    state,
                    scancode,
                    virtual_keycode,
                    shift,
                    ctrl,
                    alt,
                    logo,
                }
            },
            action,
        )
    }

    pub fn insert(&mut self, key: KeyState, action: String) {
        let node = self
            .key_tree
            .find_or_insert(KeyMap(key, HashSet::new()), |a, b| a.subsumes(&b.0));
        self.key_tree[node].value.1.insert(action);
    }

    fn attr_to_scan(attr: &Attribute, tree: &Tree<Stone>) -> Option<ScanCode> {
        match attr {
            Attribute::String(s) => match s.to_lowercase().as_str() {
                "*" => None,
                n => Some(n.parse().unwrap()),
            },
            Attribute::Select(s) => panic!("Not Implemented"),
        }
    }

    fn attr_to_virtual(attr: &Attribute, tree: &Tree<Stone>) -> Option<VirtualKeyCode> {
        match attr {
            Attribute::String(s) => match s.to_lowercase().as_str() {
                "*" => None,
                _ => panic!("Not Implemented"),
            },
            Attribute::Select(s) => panic!("Not Implemented"),
        }
    }

    fn attr_to_state(attr: &Attribute, tree: &Tree<Stone>) -> Option<ElementState> {
        match attr {
            Attribute::String(s) => match s.to_lowercase().as_str() {
                "pressed" => Some(ElementState::Pressed),
                "released" => Some(ElementState::Released),
                "*" => None,
                _ => panic!("Invalid ElementState"),
            },
            Attribute::Select(s) => panic!("Not Implemented"),
        }
    }

    fn attr_to_mod(attr: &Attribute, tree: &Tree<Stone>) -> Option<bool> {
        match attr {
            Attribute::String(s) => match s.to_lowercase().as_str() {
                "*" => None,
                _ => Some(architect::str_to_bool(s)),
            },
            Attribute::Select(s) => panic!("Not Implemented"),
        }
    }
}

impl StoneMason for ControlMason {
    fn handle_stones(
        &mut self,
        arch: &mut Architect,
        map: &mut HashMap<String, Vec<usize>>,
    ) -> HashSet<usize> {
        let mut res = HashSet::new();
        if !map.contains_key("control") {
            return res;
        }
        for c in map.get("control").unwrap() {
            let con = &arch.stones[*c];
            let action = match con.value.as_el().attr.get("action") {
                Some(a) => String::from_attr(a, &arch.stones, *c).unwrap(),
                None => {
                    arch.errors.push(StoneError::MissingAttr("action".into()));
                    continue;
                }
            };
            for leaf in con.leaves() {
                if let Stone::Element(ref el) = arch.stones[*leaf].value {
                    if el.class == "key" {
                        self.read_key(&arch, el, action.clone());
                    }
                }
            }
            res.insert(*c);
        }

        res
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
