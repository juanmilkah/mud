use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Write, stdin},
    path::PathBuf,
};

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Filepath to csv file.
    /// If missing, read from stdin
    filepath: Option<PathBuf>,
    /// Sub-command to process the data
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Sort data by category
    Sort {
        /// Sort data by Column name
        #[arg(value_name = "CATEGORY")]
        category: String,

        /// Output the first (count) lines
        #[arg(short, long)]
        count: Option<usize>,

        /// Output the result in reverse order
        #[arg(short, long, action)]
        reverse: bool,

        /// Output filepath
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Filter data by a criterion
    Filter {
        #[arg(value_name = "CATEGORY")]
        category: String,

        /// Comparison operator
        #[arg(value_name = "OPERATOR")]
        operator: Operator,

        /// Compare against value
        #[arg(value_name = "VALUE")]
        argument: f32,

        /// Output the first (count) lines
        #[arg(short, long)]
        count: Option<usize>,

        /// Output the result in reverse order
        #[arg(short, long, action)]
        reverse: bool,

        /// Output filepath
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Calculate The Mean
    Mean {
        #[arg(value_name = "CATEGORIES")]
        categories: Option<Vec<String>>,
        /// Exclude a Column
        #[arg(short = 'x', long)]
        exclude: Option<Vec<String>>,

        /// Output filepath
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    Median {
        /// Compute the median of whole | categories of data
        #[arg(value_name = "CATEGORIES")]
        categories: Option<Vec<String>>,

        /// Exclude a Column
        #[arg(short = 'x', long)]
        exclude: Option<Vec<String>>,

        /// Output filepath
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
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
    if !data.is_empty() && headers.len() != data[0].len() {
        eprintln!(
            "Header columns count does not match the data columns count: {} -> {}",
            headers.len(),
            data[0].len()
        );

        return;
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

    if rows_as_string.is_empty() {
        println!("EMPTY!");
        println!("{separator}");
        return;
    }

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

fn find_index<T: Eq>(arr: &[T], elem: &T) -> Option<usize> {
    arr.iter().position(|item| item == elem)
}

fn dump_to_file(headers: &[String], data: &[Vec<f32>], filepath: PathBuf) -> io::Result<()> {
    let file = File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open(filepath)?;
    let mut file = BufWriter::new(file);
    let headers = headers.join(",");
    writeln!(file, "{}", headers)?;
    let data = data
        .iter()
        .map(|row| {
            row.iter()
                .map(|elem| elem.to_string())
                .collect::<Vec<String>>()
                .join(",")
        })
        .collect::<Vec<String>>()
        .join("\n");

    file.write_all(data.as_bytes())?;
    Ok(())
}

fn output_result(
    data: &[Vec<f32>],
    headers: &[String],
    output: Option<PathBuf>,
) -> Result<(), String> {
    match output {
        Some(file) => {
            dump_to_file(headers, data, file).map_err(|err| format!("Save to file failed: {}", err))
        }
        None => {
            tabulate_data(data, headers);
            Ok(())
        }
    }
}

fn apply_count_and_reverse(data: &mut Vec<Vec<f32>>, count: Option<usize>, reverse: bool) {
    if reverse {
        data.reverse();
    }
    if let Some(count) = count {
        data.truncate(count);
    }
}

fn handle_sort(
    mut data: Vec<Vec<f32>>,
    headers: &[String],
    category: &str,
    count: Option<usize>,
    reverse: bool,
    output: Option<PathBuf>,
) -> Result<(), String> {
    let cat_index = find_index(headers, &category.to_lowercase())
        .ok_or_else(|| "Invalid category".to_string())?;

    if reverse {
        data.sort_by(|a, b| b[cat_index].total_cmp(&a[cat_index]));
    } else {
        data.sort_by(|a, b| a[cat_index].total_cmp(&b[cat_index]));
    }

    if let Some(count) = count {
        data.truncate(count);
    }

    output_result(&data, headers, output)
}

#[allow(clippy::too_many_arguments)]
fn handle_filter(
    data: Vec<Vec<f32>>,
    headers: &[String],
    category: &str,
    operator: &Operator,
    argument: f32,
    count: Option<usize>,
    reverse: bool,
    output: Option<PathBuf>,
) -> Result<(), String> {
    let cat_index = find_index(headers, &category.to_lowercase())
        .ok_or_else(|| "Invalid category".to_string())?;

    let mut filtered_data: Vec<Vec<f32>> = data
        .into_iter()
        .filter(|row| match operator {
            Operator::Gt => row[cat_index] > argument,
            Operator::Lt => row[cat_index] < argument,
            Operator::Eq => (row[cat_index] - argument).abs() < f32::EPSILON,
            Operator::Neq => (row[cat_index] - argument).abs() > f32::EPSILON,
            Operator::Gte => row[cat_index] >= argument,
            Operator::Lte => row[cat_index] <= argument,
        })
        .collect();

    apply_count_and_reverse(&mut filtered_data, count, reverse);
    output_result(&filtered_data, headers, output)
}

fn get_valid_categories(
    categories: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    headers: &[String],
) -> Vec<String> {
    let exclude = exclude.unwrap_or_default();

    let valid_categories = match categories {
        Some(cats) if !cats.is_empty() => cats,
        _ => headers.to_vec(),
    };

    valid_categories
        .into_iter()
        .filter(|cat| headers.contains(cat))
        .filter(|cat| !exclude.contains(cat))
        .collect()
}

fn handle_mean(
    data: &[Vec<f32>],
    headers: &[String],
    categories: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    output: Option<PathBuf>,
) -> Result<(), String> {
    let valid_categories = get_valid_categories(categories, exclude, headers);

    if valid_categories.is_empty() {
        return Err("No valid categories passed".to_string());
    }

    let row_count = data.len() as f32;
    let cat_indices: Vec<usize> = valid_categories
        .iter()
        .map(|cat| find_index(headers, cat).unwrap())
        .collect();

    let means: Vec<f32> = cat_indices
        .iter()
        .map(|&idx| data.iter().map(|row| row[idx]).sum::<f32>() / row_count)
        .collect();

    output_result(&[means], &valid_categories, output)
}

fn handle_median(
    mut data: Vec<Vec<f32>>,
    headers: &[String],
    categories: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    output: Option<PathBuf>,
) -> Result<(), String> {
    let valid_categories = get_valid_categories(categories, exclude, headers);

    if valid_categories.is_empty() {
        return Err("No valid categories passed".to_string());
    }

    let cat_indices: Vec<usize> = valid_categories
        .iter()
        .map(|cat| find_index(headers, cat).unwrap())
        .collect();

    // Sort data by each column for median calculation
    for &idx in &cat_indices {
        data.sort_by(|a, b| a[idx].total_cmp(&b[idx]));
    }

    let row_count = data.len();
    let medians: Vec<f32> = if row_count % 2 == 1 {
        let mid = row_count / 2;
        cat_indices.iter().map(|&idx| data[mid][idx]).collect()
    } else {
        let (lower, upper) = (row_count / 2 - 1, row_count / 2);
        cat_indices
            .iter()
            .map(|&idx| (data[lower][idx] + data[upper][idx]) / 2.0)
            .collect()
    };

    output_result(&[medians], &valid_categories, output)
}

fn main() -> Result<(), String> {
    let args = Cli::parse();

    let content = match args.filepath {
        Some(filepath) => {
            let file = File::open(filepath).map_err(|err| format!("File is missing: {}", err))?;
            let file = BufReader::new(file);
            io::read_to_string(file).map_err(|err| format!("Read from file failed: {}", err))?
        }
        None => {
            let stdin = stdin();
            io::read_to_string(stdin).map_err(|err| format!("Read from stdin failed: {}", err))?
        }
    };

    let headers = content
        .lines()
        .next()
        .ok_or_else(|| "Missing headers".to_string())?
        .split(",")
        .map(|s| s.trim().to_lowercase())
        .collect::<Vec<String>>();

    let data = content
        .lines()
        .skip(1)
        .filter(|&line| (!line.is_empty()))
        .map(|line| {
            line.split(",")
                .map(|elem| elem.trim())
                .map(|elem| {
                    elem.parse::<f32>()
                        .unwrap_or_else(|_| elem.parse::<i32>().unwrap_or(-1) as f32)
                })
                .collect::<Vec<f32>>()
        })
        .collect::<Vec<Vec<f32>>>();

    if headers.len() != data.len() {
        return Err("Mismatch between header count and data columns".to_string());
    }

    match args.command {
        Command::Sort {
            category,
            count,
            reverse,
            output,
        } => handle_sort(data, &headers, &category, count, reverse, output),
        Command::Filter {
            category,
            operator,
            argument,
            count,
            reverse,
            output,
        } => handle_filter(
            data, &headers, &category, &operator, argument, count, reverse, output,
        ),
        Command::Mean {
            categories,
            exclude,
            output,
        } => handle_mean(&data, &headers, categories, exclude, output),
        Command::Median {
            categories,
            exclude,
            output,
        } => handle_median(data, &headers, categories, exclude, output),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn large_dataset() -> (Vec<String>, Vec<Vec<f32>>) {
        let headers = vec!["score".to_string(), "age".to_string()];
        let data = (1..=100)
            .map(|i| vec![i as f32 * 0.5, 20.0 + (i % 50) as f32])
            .collect();
        (headers, data)
    }

    #[test]
    fn test_dump_to_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.csv");

        let headers = vec!["a".to_string(), "b".to_string()];
        let data = vec![vec![1.0, 2.0], vec![3.0, 4.0]];

        let result = dump_to_file(&headers, &data, file_path.clone());
        assert!(result.is_ok());

        let content = fs::read_to_string(file_path).unwrap();
        assert!(content.contains("a,b"));
        assert!(content.contains("1,2"));
        assert!(content.contains("3,4"));
    }

    #[test]
    fn test_large_dataset_performance() {
        let (headers, data) = large_dataset();

        // Test that operations complete on larger datasets
        let start = std::time::Instant::now();
        let result = handle_sort(data.clone(), &headers, "score", None, false, None);
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration.as_millis() < 1000); // Should complete within 1 second

        let result = handle_mean(&data, &headers, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_case_single_row() {
        let headers = vec!["value".to_string()];
        let data = vec![vec![42.0]];

        assert!(handle_sort(data.clone(), &headers, "value", None, false, None).is_ok());
        assert!(
            handle_filter(
                data.clone(),
                &headers,
                "value",
                &Operator::Eq,
                42.0,
                None,
                false,
                None
            )
            .is_ok()
        );
        assert!(handle_mean(&data, &headers, None, None, None).is_ok());
        assert!(handle_median(data, &headers, None, None, None).is_ok());
    }

    #[test]
    fn test_edge_case_negative_values() {
        let headers = vec!["temp".to_string()];
        let data = vec![vec![-10.5], vec![0.0], vec![-5.2], vec![15.3]];

        assert!(handle_sort(data.clone(), &headers, "temp", None, false, None).is_ok());
        assert!(
            handle_filter(
                data.clone(),
                &headers,
                "temp",
                &Operator::Lt,
                0.0,
                None,
                false,
                None
            )
            .is_ok()
        );
        assert!(handle_mean(&data, &headers, None, None, None).is_ok());
        assert!(handle_median(data, &headers, None, None, None).is_ok());
    }

    #[test]
    fn test_all_operators() {
        let headers = vec!["value".to_string()];
        let data = vec![vec![10.0], vec![20.0], vec![30.0]];

        let operators = vec![
            Operator::Gt,
            Operator::Gte,
            Operator::Lt,
            Operator::Lte,
            Operator::Eq,
            Operator::Neq,
        ];

        for op in operators {
            let result = {
                let data = data.clone();
                let headers: &[String] = &headers;
                let operator: &Operator = &op;
                let argument = 20.0;
                let cat_index = find_index(headers, &"value".to_lowercase()).unwrap();

                let mut filtered_data: Vec<Vec<f32>> = data
                    .into_iter()
                    .filter(|row| match operator {
                        Operator::Gt => row[cat_index] > argument,
                        Operator::Lt => row[cat_index] < argument,
                        Operator::Eq => row[cat_index] == argument,
                        Operator::Neq => row[cat_index] != argument,
                        Operator::Gte => row[cat_index] >= argument,
                        Operator::Lte => row[cat_index] <= argument,
                    })
                    .collect();

                apply_count_and_reverse(&mut filtered_data, None, false);
                output_result(&filtered_data, headers, None)
            };
            assert!(result.is_ok(), "Failed for operator: {:?}", op);
        }
    }
}
