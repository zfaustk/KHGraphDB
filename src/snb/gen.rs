//! Deterministic social graph. Scale is people.
//! Names, cities, years are tables. Edges are
//! KNOWS, LIKES, HAS_CREATOR, HAS_MEMBER,
//! CONTAINER_OF, REPLY_OF. Seed is a LCG.

use crate::graph::Graph;
use crate::khid::Khid;
use crate::prop::Prop;
use std::collections::HashMap;

const FIRST: &'static [&'static str] = &[
    "Ada", "Alan", "Grace", "Donald", "Edsger", "Barbara", "John", "Ken",
    "Dennis", "Brian", "Frances", "Leslie", "Tony", "Niklaus", "Anita",
    "Linus", "Bjarne", "Guido", "James", "Anders", "Brendan", "Tim",
    "Larry", "Sergey", "Vint", "Radia", "Margaret", "Jean", "Claude",
    "Alonzo", "Emil", "Kurt", "Haskell", "Peter", "Robert", "David",
    "Mary", "Linda", "Patricia", "Jennifer", "Elizabeth", "Susan",
    "Michael", "William", "Richard", "Joseph", "Thomas", "Charles",
    "Christopher", "Daniel", "Matthew", "Anthony", "Mark", "Steven",
    "Paul", "Andrew", "Joshua", "Kenneth", "Kevin", "Brian",
    "George", "Timothy", "Ronald", "Edward", "Jason", "Jeffrey",
    "Ryan", "Jacob", "Gary", "Nicholas", "Eric", "Jonathan",
    "Stephen", "Larry", "Justin", "Scott", "Brandon", "Benjamin",
    "Samuel", "Raymond", "Gregory", "Frank", "Alexander", "Patrick",
    "Jack", "Dennis", "Jerry", "Tyler", "Aaron", "Jose",
    "Henry", "Adam", "Douglas", "Nathan", "Peter", "Zachary",
    "Kyle", "Walter", "Harold", "Jeremy", "Ethan", "Carl",
    "Keith", "Roger", "Gerald", "Christian", "Terry", "Sean",
    "Arthur", "Austin", "Noah", "Lawrence", "Jesse", "Joe",
    "Bryan", "Billy", "Jordan", "Albert", "Dylan", "Bruce",
    "Willie", "Gabriel", "Joe", "Logan", "Alan", "Juan",
    "Wayne", "Roy", "Ralph", "Randy", "Eugene", "Vincent",
    "Russell", "Louis", "Philip", "Bobby", "Johnny", "Bradley",
];

const LAST: &'static [&'static str] = &[
    "Lovelace", "Turing", "Hopper", "Knuth", "Dijkstra", "Liskov",
    "McCarthy", "Thompson", "Ritchie", "Kernighan", "Allen", "Lamport",
    "Hoare", "Wirth", "Borg", "Torvalds", "Stroustrup", "van Rossum",
    "Gosling", "Hejlsberg", "Eich", "Berners-Lee", "Page", "Brin",
    "Cerf", "Perlman", "Hamilton", "Sammet", "Shannon", "Church",
    "Post", "Godel", "Curry", "Naur", "Floyd", "Wheeler",
    "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia",
    "Miller", "Davis", "Rodriguez", "Martinez", "Hernandez", "Lopez",
    "Gonzalez", "Wilson", "Anderson", "Thomas", "Taylor", "Moore",
    "Jackson", "Martin", "Lee", "Perez", "Thompson", "White",
    "Harris", "Sanchez", "Clark", "Ramirez", "Lewis", "Robinson",
    "Walker", "Young", "Allen", "King", "Wright", "Scott",
    "Torres", "Nguyen", "Hill", "Flores", "Green", "Adams",
    "Nelson", "Baker", "Hall", "Rivera", "Campbell", "Mitchell",
    "Carter", "Roberts", "Gomez", "Phillips", "Evans", "Turner",
    "Diaz", "Parker", "Cruz", "Edwards", "Collins", "Reyes",
    "Stewart", "Morris", "Morales", "Murphy", "Cook", "Rogers",
    "Gutierrez", "Ortiz", "Morgan", "Cooper", "Peterson", "Bailey",
    "Reed", "Kelly", "Howard", "Ramos", "Kim", "Cox",
    "Ward", "Richardson", "Watson", "Brooks", "Chavez", "Wood",
    "James", "Bennett", "Gray", "Mendoza", "Ruiz", "Hughes",
    "Price", "Alvarez", "Castillo", "Sanders", "Patel", "Myers",
];

const CITY: &'static [&'static str] = &[
    "London", "Cambridge", "Amsterdam", "Zurich", "Boston", "Stanford",
    "Munich", "Paris", "Tokyo", "Beijing", "Singapore", "Stockholm",
    "Helsinki", "Oslo", "Copenhagen", "Vienna", "Prague", "Warsaw",
    "Seoul", "Sydney", "Toronto", "Montreal", "Austin", "Seattle",
    "Portland", "Chicago", "Berlin", "Hamburg", "Madrid", "Barcelona",
    "Milan", "Rome", "Lisbon", "Dublin", "Edinburgh", "Oxford",
    "Manchester", "Bristol", "Lyon", "Toulouse", "Rotterdam", "Utrecht",
    "Basel", "Geneva", "Osaka", "Kyoto", "Taipei", "Hong Kong",
    "Bangalore", "Hyderabad", "Pune", "Mumbai", "Delhi", "Shenzhen",
    "Hangzhou", "Nanjing", "Melbourne", "Auckland", "Vancouver", "Ottawa",
];

const TAG: &'static [&'static str] = &[
    "graph", "db", "rust", "os", "net", "lang", "ai", "type",
    "logic", "algebra", "music", "photo", "climb", "run", "chess",
    "go", "unix", "lisp", "ml", "sql", "wal", "index", "query",
];

pub struct Rng {
    s: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng { s: if seed == 0 { 1 } else { seed } }
    }

    pub fn next(&mut self) -> u64 {
        self.s = self.s.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.s
    }

    pub fn under(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next() as usize) % n
        }
    }

    pub fn between(&mut self, lo: i64, hi: i64) -> i64 {
        if hi <= lo {
            lo
        } else {
            lo + (self.next() as i64).abs() % (hi - lo)
        }
    }
}

fn put(g: &mut Graph, name: &str, first: &str, last: &str, city: &str, year: i64) -> Khid {
    let mut a = HashMap::new();
    a.insert("name".to_string(), Prop::from_str(name));
    a.insert("first".to_string(), Prop::from_str(first));
    a.insert("last".to_string(), Prop::from_str(last));
    a.insert("city".to_string(), Prop::from_str(city));
    a.insert("year".to_string(), Prop::from_int(year));
    g.add_vertex_props(a, Some("Person")).unwrap()
}

fn post(g: &mut Graph, title: &str, year: i64) -> Khid {
    let mut a = HashMap::new();
    a.insert("name".to_string(), Prop::from_str(title));
    a.insert("title".to_string(), Prop::from_str(title));
    a.insert("year".to_string(), Prop::from_int(year));
    g.add_vertex_props(a, Some("Post")).unwrap()
}

fn forum(g: &mut Graph, title: &str) -> Khid {
    let mut a = HashMap::new();
    a.insert("name".to_string(), Prop::from_str(title));
    a.insert("title".to_string(), Prop::from_str(title));
    g.add_vertex_props(a, Some("Forum")).unwrap()
}

pub struct Social {
    pub g: Graph,
    pub people: Vec<Khid>,
    pub posts: Vec<Khid>,
    pub forums: Vec<Khid>,
    pub knows: usize,
    pub likes: usize,
}

/// `n` people. ~2n posts, n/4 forums, ~4n KNOWS, ~3n LIKES.
pub fn generate(n: usize, seed: u64) -> Social {
    let mut g = Graph::named("snb");
    g.create_index("Person", "name");
    g.create_index("Person", "last");
    g.create_index("Person", "city");
    g.create_index("Post", "name");
    g.create_index("Forum", "name");
    let mut rng = Rng::new(seed);
    let mut people = Vec::new();
    let mut i = 0;
    while i < n {
        let f = FIRST[i % FIRST.len()];
        let l = LAST[(i / 3 + rng.under(LAST.len())) % LAST.len()];
        let city = CITY[rng.under(CITY.len())];
        let year = rng.between(1950, 2001);
        let name = format!("{}{}", f, i);
        people.push(put(&mut g, &name, f, l, city, year));
        i += 1;
    }
    let nforum = (n / 4).max(1);
    let mut forums = Vec::new();
    i = 0;
    while i < nforum {
        let title = format!("forum{}", i);
        let fid = forum(&mut g, &title);
        forums.push(fid);
        let members = 4 + rng.under(8);
        let mut j = 0;
        while j < members && j < n {
            let p = people[(i * 3 + j) % n];
            g.add_edge(fid, p, Some("HAS_MEMBER")).unwrap();
            j += 1;
        }
        i += 1;
    }
    let npost = n * 2;
    let mut posts = Vec::new();
    i = 0;
    while i < npost {
        let title = format!("p{}", i);
        let year = rng.between(2010, 2023);
        let pid = post(&mut g, &title, year);
        posts.push(pid);
        let author = people[rng.under(n)];
        g.add_edge(pid, author, Some("HAS_CREATOR")).unwrap();
        let fo = forums[rng.under(forums.len())];
        g.add_edge(fo, pid, Some("CONTAINER_OF")).unwrap();
        if i > 0 && rng.under(4) == 0 {
            let parent = posts[rng.under(i)];
            g.add_edge(pid, parent, Some("REPLY_OF")).unwrap();
        }
        i += 1;
    }
    let mut knows = 0usize;
    i = 0;
    while i < n {
        let deg = 2 + rng.under(5);
        let mut j = 0;
        while j < deg {
            let o = rng.under(n);
            if o != i {
                g.add_edge(people[i], people[o], Some("KNOWS")).unwrap();
                knows += 1;
            }
            j += 1;
        }
        i += 1;
    }
    let mut likes = 0usize;
    i = 0;
    while i < n {
        let k = 1 + rng.under(4);
        let mut j = 0;
        while j < k {
            let p = posts[rng.under(posts.len())];
            g.add_edge(people[i], p, Some("LIKES")).unwrap();
            likes += 1;
            j += 1;
        }
        i += 1;
    }
    let ntag = TAG.len().min(8 + n / 20);
    i = 0;
    while i < ntag {
        let mut a = HashMap::new();
        a.insert("name".to_string(), Prop::from_str(TAG[i % TAG.len()]));
        let t = g.add_vertex_props(a, Some("Tag")).unwrap();
        let m = 2 + rng.under(6);
        let mut j = 0;
        while j < m {
            g.add_edge(posts[rng.under(posts.len())], t, Some("HAS_TAG")).unwrap();
            j += 1;
        }
        i += 1;
    }
    Social {
        g: g,
        people: people,
        posts: posts,
        forums: forums,
        knows: knows,
        likes: likes,
    }
}
