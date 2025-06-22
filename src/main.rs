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
    /// Output the first (count) lines
    #[arg(short, long)]
    count: Option<usize>,
    /// Output the result in reverse order
    #[arg(short, long, action)]
    reverse: bool,
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
    /// Calculate The Mean
    Mean {
        #[arg(value_name = "CATEGORY")]
        categories: Option<Vec<String>>,
        /// Exclude a Column
        #[arg(short = 'x', long)]
        exclude: Option<Vec<String>>,
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

fn tabulate_data(data: &[Vec<f32>], headers: &[String]) {
    // tabulate the data
    // *------------------------------*
    // * id    * price    * amount    *
    // *-------*----------*-----------*
    // * 1     * 10.50    * 5         *
    // * 2     * 25       * 10        *
    //

    let header_cols = headers.len();
    let data_cols = data[0].len();
    if !data.is_empty() && header_cols != data_cols {
        eprintln!(
            "Header columns count does not match the data columns count: {} -> {}",
            header_cols, data_cols
        );
        std::process::exit(1);
    }

    let rows_as_string: Vec<Vec<String>> = data
        .iter()
        .map(|row| row.iter().map(|elem| format!("{:.2}", elem)).collect())
        .collect();

    let mut cols_widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in &rows_as_string {
        for (i, elem) in row.iter().enumerate() {
            cols_widths[i] = cols_widths[i].max(elem.len());
        }
    }

    // headers separator
    let separator = cols_widths
        .iter()
        .map(|&w| "=".repeat(w + 2))
        .collect::<Vec<_>>()
        .join("*");
    println!("{separator}");

    let headers_row = headers
        .iter()
        .enumerate()
        .map(|(i, h)| format!(" {:>width$} ", h, width = cols_widths[i]))
        .collect::<Vec<_>>()
        .join("*");

    println!("{headers_row}");
    println!("{separator}");

    for row in rows_as_string {
        let row = row
            .iter()
            .enumerate()
            .map(|(i, elem)| format!(" {:>width$} ", elem, width = cols_widths[i]))
            .collect::<Vec<_>>()
            .join("*");
        println!("{row}");
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

fn main() -> Result<(), String> {
    let args = Cli::parse();

    let mut content = String::new();
    let file = File::open(args.filepath).map_err(|err| format!("File is missing: {}", err))?;
    let mut file = BufReader::new(file);
    file.read_to_string(&mut content)
        .map_err(|err| format!("failed to read from file: {}", err))?;

    let headers = content
        .lines()
        .next()
        .ok_or_else(|| "Missing headers".to_string())?
        .split(",")
        .map(|s| s.trim().to_lowercase())
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
                            .unwrap_or_else(|_| elem.parse::<i32>().unwrap_or(-1) as f32)
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

            let cat_index =
                index_of(&headers, &category).ok_or_else(|| "category not found".to_string())?;
            if args.reverse {
                cleaned_data.sort_by(|a, b| b[cat_index].total_cmp(&a[cat_index]));
            } else {
                cleaned_data.sort_by(|a, b| a[cat_index].total_cmp(&b[cat_index]));
            }
            if let Some(count) = args.count {
                cleaned_data.truncate(count);
            }
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

            let cat_index =
                index_of(&headers, &category).ok_or_else(|| "category not found".to_string())?;
            let mut processed_data = cleaned_data
                .into_iter()
                .filter(|row| match instruction {
                    Operator::Gt => row[cat_index] > argument,
                    Operator::Lt => row[cat_index] < argument,
                    Operator::Eq => (row[cat_index] - argument).abs() < f32::EPSILON,
                    Operator::Neq => (row[cat_index] - argument).abs() > f32::EPSILON,
                    Operator::Gte => row[cat_index] >= argument,
                    Operator::Lte => row[cat_index] <= argument,
                })
                .collect::<Vec<Vec<f32>>>();
            if args.reverse {
                processed_data.reverse();
            }
            if let Some(count) = args.count {
                processed_data.truncate(count);
            }
            tabulate_data(&processed_data, &headers);
        }

        Command::Mean {
            categories,
            exclude,
        } => {
            if categories.is_none() || categories.as_ref().is_some_and(|list| list.is_empty()) {
                let mut skips = Vec::new();
                if let Some(exclude) = exclude {
                    for col in exclude {
                        if headers.contains(&col) {
                            if let Some(idx) = index_of(&headers, &col) {
                                skips.push(idx)
                            }
                        }
                    }
                }

                let row_count = cleaned_data.len();
                let mut sums: Vec<f32> = vec![0.0; headers.len() - skips.len()];
                for row in cleaned_data.into_iter() {
                    let mut i = 0;
                    for (col, elem) in row.iter().enumerate() {
                        if !skips.contains(&col) {
                            sums[i] += elem;
                            i += 1;
                        }
                    }
                }
                let means = sums
                    .into_iter()
                    .map(|total| total / row_count as f32)
                    .collect::<Vec<f32>>();
                let skips = skips
                    .into_iter()
                    .map(|i| &headers[i])
                    .cloned()
                    .collect::<Vec<String>>();
                let headers = headers
                    .into_iter()
                    .filter(|h| !skips.contains(h))
                    .collect::<Vec<String>>();

                tabulate_data(&[means], &headers);
                return Ok(());
            }

            let categories: Vec<String> = categories
                .unwrap()
                .iter()
                .filter(|cat| headers.contains(cat))
                .cloned()
                .collect();
            if categories.is_empty() {
                eprintln!("No valid categories passed");
                std::process::exit(1);
            }
            let mut means = Vec::with_capacity(categories.len());

            for category in &categories {
                let cat_index =
                    index_of(&headers, category).ok_or_else(|| "category not found".to_string())?;
                let values: Vec<f32> = cleaned_data.iter().map(|row| row[cat_index]).collect();
                let mean = values.iter().sum::<f32>() / values.len() as f32;
                means.push(mean);
            }
            tabulate_data(&[means], &categories);
        }
    }

    std::process::exit(0);
}
