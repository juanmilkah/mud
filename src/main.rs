use std::{
    env,
    fs::File,
    io::{BufReader, Read},
};

fn visualize_data(data: &[Vec<f32>], headers: &[String]) {
    // tabulate the data
    // *------------------------------*
    // * id    * price    * amount    *
    // *-------*----------*-----------*
    // * 1     * 10.50    * 5         *
    // * 2     * 25       * 10        *
    //

    if !data.is_empty() && headers.len() != data[0].len() {
        eprintln!("Headers length rows not match data columns");
        std::process::exit(1);
    }

    let rows_as_string: Vec<Vec<String>> = data
        .iter()
        .map(|row| row.iter().map(|elem| format!("{:.2}", elem)).collect())
        .collect();

    let mut col_widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in &rows_as_string {
        for (i, elem) in row.iter().enumerate() {
            col_widths[i] = col_widths[i].max(elem.len());
        }
    }

    // headers separator
    let separator = col_widths
        .iter()
        .map(|&w| "-".repeat(w + 2))
        .collect::<Vec<_>>()
        .join("*");
    println!("{separator}");

    let headers_row = headers
        .iter()
        .enumerate()
        .map(|(i, h)| format!(" {:^width$} ", h, width = col_widths[i]))
        .collect::<Vec<_>>()
        .join("*");

    println!("*{headers_row}*");
    println!("{separator}");

    for row in rows_as_string {
        let row = row
            .iter()
            .enumerate()
            .map(|(i, elem)| format!(" {:^width$} ", elem, width = col_widths[i]))
            .collect::<Vec<_>>()
            .join("*");
        println!("*{row}*");
    }

    println!("{separator}");
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
                .map(|elem| {
                    elem.parse::<f32>()
                        .unwrap_or_else(|_| elem.parse::<u32>().expect("Malformed data") as f32)
                })
                .collect::<Vec<f32>>()
        })
        .collect::<Vec<Vec<f32>>>();

    let command = args.next().expect("Missing command");
    match command.as_str() {
        "--sort" => {
            let category = args.next().expect("missing sort category");
            if !headers.contains(category) {
                eprintln!("Invalid category");
                std::process::exit(1);
            }

            let cat_index = index_of(&headers, category).expect("Category not found");
            cleaned_data.sort_by(|a, b| a[cat_index].total_cmp(&b[cat_index]));
            visualize_data(&cleaned_data, &headers);
            std::process::exit(0);
        }
        _ => {
            eprintln!("Invalid command");
            std::process::exit(1);
        }
    }
}
