use anyhow::Result;

mod config;
mod history;

pub use config::Config;
pub use history::History;

pub trait SyncFile {
    fn modified_time(&self) -> &std::time::SystemTime;
    fn path(&self) -> &std::path::Path;
    fn merge(&mut self, on_disk: String) -> Result<()>;
    fn write_back(&mut self) -> Result<()>;

    fn sync(&mut self) -> Result<()> {
        let disk_access_time = self.path().metadata()?.modified()?;
        match self.modified_time().duration_since(disk_access_time) {
            // Ok means file has not been modified since our last accessed
            Ok(_) => self.write_back()?,
            // Err means file has benn modified since out last accesed
            Err(_) => {
                let on_disk = std::fs::File::open(self.path())?;
                let on_disk = std::io::read_to_string(on_disk)?;
                self.merge(on_disk)?;
                self.write_back()?;
            }
        }

        Ok(())
    }
}
