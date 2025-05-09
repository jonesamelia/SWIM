use std::collections::HashMap;
use std::io::{self, Write};

const RED: &str     = "\x1b[31m";
const GREEN: &str   = "\x1b[32m";
const BLUE: &str    = "\x1b[34m";
const CYAN: &str    = "\x1b[36m";
const RESET: &str   = "\x1b[0m";

struct Room {
    description:String,
    exits: HashMap<String,usize>,
    items: Vec<String>,
    enemies: Vec<Enemy>
}

struct Enemy {
    name: String,
    hp: i32,
}

struct Game {
    rooms: Vec<Room>,
    current_room: usize,
    inventory: Vec<String>
}

impl Game {
    fn new() -> Game {
        let mut rooms = Vec::new();
        
        let mut r0_exits = HashMap::new(); r0_exits.insert("north".to_string(),1);
        rooms.push(Room{
            description: "You are in a small, dimly lit room. There is a door to the north.".to_string(),
            exits: r0_exits,
            items: vec!["flute".to_string()],
            enemies: Vec::new(),
        });
        

        let mut r1_exits = HashMap::new(); r1_exits.insert("south".to_string(),0); r1_exits.insert("east".to_string(),2);
        rooms.push(Room{
            description: "You are in a long corridor. A nasty goblin blocks your path.".to_string(),
            exits: r1_exits,
            items: Vec::new(),
            enemies: vec![ Enemy{name:"goblin".to_string(), hp: 10} ],
        });
        
        let mut r2_exits = HashMap::new(); r2_exits.insert("south".to_string(),3);
        rooms.push(Room{
            description: "You are in a treasure room! A shiny chest sits here.".to_string(),
            exits: r2_exits,
            items: vec!["treasure".to_string()],
            enemies: Vec::new(),
        });

        let mut r3_exits = HashMap::new(); r3_exits.insert("north".to_string(),2);
        rooms.push(Room{
            description: "You see a large door to the outside! It is blocked by a large dragon...".to_string(),
            exits: r3_exits,
            items: vec!["treasure".to_string()],
            enemies: vec![ Enemy{name:"dragon".to_string(), hp:50}],
        });

        Game{ rooms, current_room:0, inventory:Vec::new() }
    }

    fn play(&mut self) {

        println!("{}You awaken in a small, dark room, covered in dust.{}", CYAN, CYAN);
        println!("{}How did you get here?{}", RED, RESET);
        println!("{}Type 'help' for commands.{}", BLUE, RESET);

        loop {
            let room = &self.rooms[self.current_room];
            println!("\n{}", room.description);
            if !room.items.is_empty() {
                println!("Items here: {}", room.items.join(", "));
            }
            if !room.enemies.is_empty() {
                let names: Vec<_> = room.enemies.iter().map(|e| format!("{}(hp {})", e.name, e.hp)).collect();
                println!("Enemies: {}", names.join(", "));
            }
            print!("\n> "); io::stdout().flush().unwrap();

            let mut input=String::new(); io::stdin().read_line(&mut input).unwrap();
            let cmd = input.trim().to_lowercase();
            let mut parts = cmd.split_whitespace();

            if let Some(action)=parts.next() {
                 match action {
                    "help" => self.print_help(),
                    "go" => if let Some(d)=parts.next() 
                    { self.go(d) } 
                    else { println!("Go where?") },
                    "take" => if let Some(it)=parts.next() {
                         self.take(it) } 
                         else { println!("Take what?") 
                        },
                    "drop" => if let Some(it)=parts.next() 
                    { self.drop(it) } 
                    else { println!("Drop what?") 
                        }, "attack" => if let Some(en)=parts.next() 
                    { self.attack(en) } else {
                         println!("Attack whom?") 
                    },

                    "inventory"|"inv" => self.show_inventory(),
                    "map" => self.cmd_map(),
                    "quit"|"exit" => { println!("Have a nice day."); break },
                    _ => println!("Huh?")
                    }
            }
        }
    }

    fn print_help(&self) {
        println!("Commands:");
        println!("  help             Show this");
        println!("  go <dir>         Move");
        println!("  take <item>      Pick up");
        println!("  drop <item>      Drop");
        println!("  attack <enemy>   Attack");
        println!("  inventory/inv    Show inventory");
        println!("  quit             Exit");
        println!("  map             See map");

    }

    fn cmd_map(&self) {
       
        let layout = vec![
            vec![Some(0), Some(3)],        
            vec![Some(1), Some(2)],     
        ];
        println!("\nMap:");
        for row in (0..layout.len()).rev() {
            for col in 0..layout[row].len() {
                let cell = layout[row][col];
                let symbol = match cell {
                    Some(i) if i == self.current_room => "X",
                    Some(0) => "S",
                    Some(1) => "C",
                    Some(2) => "T",
                    Some(3) => "D",
                    _ => ".",
                };
                print!("{} ", symbol);
            }
            println!();
        }
    }
    fn go(&mut self, dir:&str) {
        let room = &self.rooms[self.current_room];
        if !room.enemies.is_empty() {
            println!("You cannot escape! You must deal with enemies first!"); return;
        }
        if let Some(&nr)=room.exits.get(dir) {
             self.current_room=nr; println!("You go {}.",dir)
        } 
        else { println!("No exit {}.",dir) }
    }

    fn take(&mut self, item:&str) {
        let rm = &mut self.rooms[self.current_room];

        if let Some(i)=rm.items.iter().position(|x|x==item) {
            self.inventory.push(item.to_string()); rm.items.remove(i); println!("Got {}.",item)
        } 
        else { 
            println!("None {} here.",item) 
        }
    }

    fn drop(&mut self, item:&str) {
        if let Some(i)=self.inventory.iter().position(|x|x==item) {
            self.rooms[self.current_room].items.push(item.to_string()); self.inventory.remove(i); println!("Dropped {}.",item)
        } else { 
            println!("You don't have {}.",item) 
        }
    }

    fn attack(&mut self, name:&str) {
        let rm = &mut self.rooms[self.current_room];
        if let Some(idx)=rm.enemies.iter().position(|e|e.name==name) {
            let mut enemy = &mut rm.enemies[idx];
            enemy.hp -= 5;
            println!("You hit {}! HP is now {}.", name, enemy.hp);
            if enemy.hp <=0 {
                println!("{} is defeated!", name);
                rm.enemies.remove(idx);
            }
        } else {
             println!("No {} here.",name) 
        }
    }

     fn show_inventory(&self) {
        if self.inventory.is_empty() { println!("Empty."); }
        else { 
            println!("Inventory: {}", self.inventory.join(", ")); 
        }
    }
}

fn main(){ let mut g=Game::new(); g.play(); }