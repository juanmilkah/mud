use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Filepath to csv file
    filepath: PathBuf,
    /// Sub-command to process the data
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Sort data by category
    Sort {
        #[arg(value_name = "CATEGORY")]
        category: String,
    },
    /// Filter data by a criterion
    Filter {
        #[arg(value_name = "CATEGORY")]
        category: String,

        #[arg(value_name = "OPERATOR")]
        operator: Operator,

        #[arg(value_name = "VALUE")]
        argument: f32,
    },
}

#[derive(Clone, ValueEnum)]
enum Operator {
    /// Greater than
    Gt,
    /// Greater than or Equal
    Gte,
    /// Less than
    Lt,
    /// Less than or Equal
    Lte,
    /// Equal to
    Eq,
    /// Not Equal to
    Neq,
}

// Support using symbols rather than words for operators
// ">" instead of "gt"
// "<" instead of "lt"
impl std::str::FromStr for Operator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ">" | "gt" => Ok(Self::Gt),
            "<" | "lt" => Ok(Self::Lt),
            "!=" | "neq" => Ok(Self::Neq),
            "=" | "eq" => Ok(Self::Eq),
            "<=" | "lte" => Ok(Self::Lte),
            ">=" | "gte" => Ok(Self::Gte),
            _ => Err(format!("Unknown operator: {}", s)),
        }
    }
}

fn tabulate_data(data: &[Vec<f32>], headers: &[String]) {
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
    let args = Cli::parse();

    let mut content = String::new();
    let mut file = BufReader::new(File::open(args.filepath).expect("File is missing"));
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
        .filter_map(|line| {
            let matches = !line.is_empty();
            matches.then(|| {
                line.split(",")
                    .map(|elem| elem.trim())
                    .map(|elem| {
                        elem.parse::<f32>()
                            .unwrap_or_else(|_| elem.parse::<u32>().expect("Malformed data") as f32)
                    })
                    .collect::<Vec<f32>>()
            })
        })
        .collect::<Vec<Vec<f32>>>();

    match args.command {
        Command::Sort { category } => {
            if !headers.contains(&category) {
                eprintln!("Invalid category");
                std::process::exit(1);
            }

            let cat_index = index_of(&headers, &category).expect("Category not found");
            cleaned_data.sort_by(|a, b| a[cat_index].total_cmp(&b[cat_index]));
            tabulate_data(&cleaned_data, &headers);
            std::process::exit(0);
        }
        Command::Filter {
            category,
            operator: instruction,
            argument,
        } => {
            if !headers.contains(&category) {
                eprintln!("Invalid category");
                std::process::exit(1);
            }

            let cat_index = index_of(&headers, &category).expect("Missing category");
            let processed_data = cleaned_data
                .into_iter()
                .filter(|row| match instruction {
                    Operator::Gt => row[cat_index] > argument,
                    Operator::Lt => row[cat_index] < argument,
                    Operator::Eq => row[cat_index] == argument,
                    Operator::Neq => row[cat_index] != argument,
                    Operator::Gte => row[cat_index] >= argument,
                    Operator::Lte => row[cat_index] <= argument,
                })
                .collect::<Vec<Vec<f32>>>();
            tabulate_data(&processed_data, &headers);
        }
    }

    std::process::exit(0);
}
