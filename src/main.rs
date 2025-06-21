use std::{
    env,
    fs::File,
    io::{BufReader, Read},
};

fn visualize_data(data: &[Vec<u32>]) {
    // tabulate the data in future
    for row in data {
        let row = row
            .iter()
            .map(|elem| elem.to_string())
            .collect::<Vec<String>>()
            .join(",");
        println!("{row}");
    }
}

fn index_of<T>(arr: &[T], elem: &T) -> Option<usize>
where
    T: Eq,
{
    for (i, item) in arr.iter().enumerate() {
        if item == elem {
            return Some(i);
        }
    }
    None
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut args = args.iter();
    args.next().unwrap();

    let filepath = args.next().expect("Missing filepath");
    let mut content = String::new();
    let mut file = BufReader::new(File::open(filepath).expect("File is missing"));
    file.read_to_string(&mut content).expect("read from file");

    let headers = content
        .lines()
        .next()
        .expect("Missing headers")
        .split(",")
        .map(|s| s.to_lowercase())
        .collect::<Vec<String>>();

    let mut cleaned_data = content
        .lines()
        .skip(1)
        .map(|line| {
            line.split(",")
                .map(|elem| elem.trim())
                .map(|elem| elem.parse::<u32>().expect("Malformed data"))
                .collect::<Vec<u32>>()
        })
        .collect::<Vec<Vec<u32>>>();

    let command = args.next().expect("Missing command");
    match command.as_str() {
        "--sort" => {
            let category = args.next().expect("missing sort category");
            if !headers.contains(category) {
                eprintln!("Invalid category");
                std::process::exit(1);
            }

            let cat_index = index_of(&headers, category).expect("Category not found");
            cleaned_data.sort_by(|a, b| a[cat_index].cmp(&b[cat_index]));
            visualize_data(&cleaned_data);
            std::process::exit(0);
        }
        _ => {
            eprintln!("Invalid command");
            std::process::exit(1);
        }
    }
}
