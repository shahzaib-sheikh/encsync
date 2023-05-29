use indicatif::ProgressBar;

pub fn show_progress_bar() {
    let pb = ProgressBar::new(100);
    for i in 0..100 {
        pb.println(format!("[+] finished #{}", i));
        pb.inc(1);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    pb.finish_with_message("done");
}
