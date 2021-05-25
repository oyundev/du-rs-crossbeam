extern crate crossbeam;
use crossbeam::atomic::AtomicCell;
use std::path::PathBuf;
use rustop::opts;

fn main() {
    let (args, _rest) = opts! {
        synopsis "Calculate directory size recursively.";
		/* opt poolsize:usize=1024,
				desc:"Set pool capacity.";
		 opt stacksize:usize=3072,
				desc:"Set stack size.";
		 opt threads:usize=4,
				desc:"Number of worker threads."; */
        param folder:Option<String>, desc:"Target directory name.";
    }.parse_or_exit();

	let mut startfolder = String::from(".");
	if let Some(folder) = args.folder { startfolder = folder; }
	let n = disk_usage(PathBuf::from(startfolder));
	println!("{:.2}M", (n as f64) / 1048576_f64);
}

fn disk_usage(currpath: PathBuf) -> u64 {
    let meta = std::fs::metadata(&currpath).unwrap();
    let file_type = meta.file_type();
    if file_type.is_dir() {
        let total = AtomicCell::new(0_u64);
        let _ = crossbeam::scope(|scope| {
            for entry in std::fs::read_dir(&currpath).unwrap() {
                let e = entry.unwrap();
                let p = e.path();
                let m = e.metadata().unwrap();
                if m.file_type().is_dir() {
					 /*  scope.builder().stack_size(2 * 1024).spawn(|_| { */
					 let _ = scope.spawn(|_| { 
						total.fetch_add(disk_usage(p));
					});
                } else {
                    total.fetch_add(m.len());
                }
            }
        });
        return total.load();
    } else if file_type.is_file() {
        return meta.len();
    } else {
        return 0;
    }
}
