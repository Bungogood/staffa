use bkgm::Position;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use staffa::onnx::OnnxEvaluator;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file
    infile: PathBuf,

    /// Output file
    outfile: PathBuf,

    /// Separator
    #[arg(short = 's', default_value = ",")]
    sep: char, // TODO: Fix this to be a single byte and accept ;
}

fn count_lines<R: io::Read>(reader: R) -> io::Result<usize> {
    let buf_reader = BufReader::new(reader);
    Ok(buf_reader.lines().count())
}

fn run(args: &Args) -> io::Result<()> {
    let evaluator = OnnxEvaluator::from_file_path("model/staffa.onnx").unwrap();

    let mut infile = File::open(&args.infile)?;
    let outfile = File::create(&args.outfile)?;

    let total_lines = count_lines(&infile)?;
    infile.seek(io::SeekFrom::Start(0))?; // Reset file position to the beginning.

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(args.sep as u8)
        .has_headers(true)
        .from_reader(BufReader::new(infile));
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(args.sep as u8)
        .from_writer(outfile);

    let pb = ProgressBar::new(total_lines as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{wide_bar} {pos}/{len} ({percent}%)")
            .unwrap(),
    );

    let mut headers: Vec<String> = rdr
        .byte_headers()?
        .iter()
        .skip(1)
        .map(|s| String::from_utf8_lossy(s).to_string())
        .collect();
    headers.extend(evaluator.input_labels());

    wtr.write_record(headers)?;

    for line in rdr.records() {
        let line = line?;
        let mut line_iter = line.iter();
        let pid = line_iter.next().unwrap();
        let position = Position::from_id(pid.to_string()).unwrap();
        let inputs = evaluator.inputs(&position);
        let mut data = line_iter.map(|f| f.to_string()).collect::<Vec<String>>();
        data.extend(inputs.iter().map(|f| f.to_string()));
        wtr.write_record(data)?;
        pb.inc(1);
    }
    pb.finish_and_clear();
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    run(&args)
}
