use oxyd_domain::errors::ProcessError;

#[derive(Debug, Clone)]
pub struct StatFields {
    pub comm: String,
    pub state: char,
    pub ppid: u32,
    pub priority: i32,
    pub nice: i32,
    pub starttime: u64,
    pub utime: u64,
    pub stime: u64,
    pub cutime: u64,
    pub cstime: u64,
}

pub struct StatusFields {
    pub uid: String,
    pub gid: String,
    pub threads: u32,
    pub vm_size: u64,
    pub rss_bytes: u64,
    pub read_bytes: u64,
    pub write_bytes: u64,
}

pub fn parse_stat(stat_content: &str) -> Result<StatFields, ProcessError> {
    let start = stat_content.find('(')
        .ok_or_else(|| ProcessError::ParseError("Invalid stat format".to_string()))?;
    let end = stat_content.rfind(')')
        .ok_or_else(|| ProcessError::ParseError("Invalid stat format".to_string()))?;

    let comm = stat_content[start + 1..end].to_string();
    let after_comm = &stat_content[end + 2..];

    let fields: Vec<&str> = after_comm.split_whitespace().collect();

    if fields.len() < 20 {
        return Err(ProcessError::ParseError("Insufficient stat fields".to_string()));
    }

    Ok(StatFields {
        comm,
        state: fields[0].chars().next().unwrap_or('?'),
        ppid: fields[1].parse().unwrap_or(0),
        priority: fields[15].parse().unwrap_or(20),
        nice: fields[16].parse().unwrap_or(0),
        starttime: fields[19].parse().unwrap_or(0),
        utime: fields[11].parse().unwrap_or(0),
        stime: fields[12].parse().unwrap_or(0),
        cutime: fields[13].parse().unwrap_or(0),
        cstime: fields[14].parse().unwrap_or(0),
    })
}

pub fn parse_status(status_content: &str) -> StatusFields {
    let mut uid = String::from("0");
    let mut gid = String::from("0");
    let mut threads = 1;
    let mut vm_size = 0;
    let mut rss_bytes = 0;
    let read_bytes = 0;
    let write_bytes = 0;

    for line in status_content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        match parts[0] {
            "Uid:" => {
                if parts.len() > 1 {
                    uid = parts[1].to_string();
                }
            }
            "Gid:" => {
                if parts.len() > 1 {
                    gid = parts[1].to_string();
                }
            }
            "Threads:" => {
                threads = parts[1].parse().unwrap_or(1);
            }
            "VmSize:" => {
                if parts.len() > 1 {
                    vm_size = parts[1].parse::<u64>().unwrap_or(0) * 1024;
                }
            }
            "VmRSS:" => {
                if parts.len() > 1 {
                    rss_bytes = parts[1].parse::<u64>().unwrap_or(0) * 1024;
                }
            }
            _ => {}
        }
    }

    StatusFields {
        uid,
        gid,
        threads,
        vm_size,
        rss_bytes,
        read_bytes,
        write_bytes,
    }
}
