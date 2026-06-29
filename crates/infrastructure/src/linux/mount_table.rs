//! Parser puro de `/proc/mounts` para achar o ponto de montagem de um device.

/// Localiza pontos de montagem de partições no `/proc/mounts`.
pub(crate) struct MountTable;

impl MountTable {
    /// Primeiro mount point cujo device é `/dev/<name>` ou `/dev/<name><dígitos>`.
    #[must_use]
    pub(crate) fn mount_point_for(contents: &str, name: &str) -> Option<String> {
        contents.lines().find_map(|line| {
            let mut fields = line.split(' ');
            let source = fields.next()?;
            let mount = fields.next()?;
            let dev = source.strip_prefix("/dev/")?;
            if Self::matches(dev, name) {
                Some(Self::decode(mount))
            } else {
                None
            }
        })
    }

    // `dev` corresponde a `name` (o próprio device ou uma partição `name<dígitos>`).
    fn matches(dev: &str, name: &str) -> bool {
        match dev.strip_prefix(name) {
            Some("") => true,
            Some(rest) => rest.bytes().all(|b| b.is_ascii_digit()),
            None => false,
        }
    }

    // Decodifica escapes octais do `/proc/mounts` (ex.: `\040` → espaço).
    fn decode(raw: &str) -> String {
        raw.replace("\\040", " ")
            .replace("\\011", "\t")
            .replace("\\134", "\\")
    }
}

#[cfg(test)]
mod tests;
