use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Command {
    pub ssh_key_path: String,
    pub user_name: String,

    pub region: String,
    pub availability_zone: String,

    pub instance_id: String,
    pub instance_state: String,

    pub ip_mode: String,
    pub public_ip: String,

    pub profile: Option<String>,
}

/// ref. <https://doc.rust-lang.org/std/string/trait.ToString.html>
/// ref. <https://doc.rust-lang.org/std/fmt/trait.Display.html>
/// Use "Self.to_string()" to directly invoke this.
impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ssh -o "StrictHostKeyChecking no" -i [ssh_key_path] [user name]@[public IPv4/DNS name]
        // aws ssm start-session --region [region] --target [instance ID]
        write!(
            f,
            "# change SSH key permission
chmod 400 {ssh_key_path}

# instance '{instance_id}' ({instance_state}, {availability_zone}) -- ip mode '{ip_mode}'
ssh -o \"StrictHostKeyChecking no\" -i {ssh_key_path} {user_name}@{public_ip}
ssh -o \"StrictHostKeyChecking no\" -i {ssh_key_path} {user_name}@{public_ip} 'tail -10 /var/log/cloud-init-output.log'
ssh -o \"StrictHostKeyChecking no\" -i {ssh_key_path} {user_name}@{public_ip} 'tail -f /var/log/cloud-init-output.log'

# download a remote file to local machine
scp -i {ssh_key_path} {user_name}@{public_ip}:REMOTE_FILE_PATH LOCAL_FILE_PATH
scp -i {ssh_key_path} -r {user_name}@{public_ip}:REMOTE_DIRECTORY_PATH LOCAL_DIRECTORY_PATH

# upload a local file to remote machine
scp -i {ssh_key_path} LOCAL_FILE_PATH {user_name}@{public_ip}:REMOTE_FILE_PATH
scp -i {ssh_key_path} -r LOCAL_DIRECTORY_PATH {user_name}@{public_ip}:REMOTE_DIRECTORY_PATH

# AWS SSM session (requires a running SSM agent)
# https://github.com/aws/amazon-ssm-agent/issues/131
aws ssm start-session {profile_flag}--region {region} --target {instance_id}
aws ssm start-session {profile_flag}--region {region} --target {instance_id} --document-name 'AWS-StartNonInteractiveCommand' --parameters command=\"sudo tail -10 /var/log/cloud-init-output.log\"
aws ssm start-session {profile_flag}--region {region} --target {instance_id} --document-name 'AWS-StartInteractiveCommand' --parameters command=\"bash -l\"
",
            ssh_key_path = self.ssh_key_path,
            user_name = self.user_name,

            region = self.region,
            availability_zone = self.availability_zone,

            instance_id = self.instance_id,
            instance_state = self.instance_state,

            ip_mode = self.ip_mode,
            public_ip = self.public_ip,

            profile_flag = if let Some(v) = &self.profile {
                format!("--profile {v} ")
            } else {
                String::new()
            },
        )
    }
}

impl Command {
    /// Run a command remotely.
    pub fn run(&self, cmd: &str) -> io::Result<command_manager::Output> {
        log::info!("sending an SSH command to {}", self.public_ip);
        let remote_cmd_to_run = format!("chmod 400 {ssh_key_path} && ssh -o \"StrictHostKeyChecking no\" -i {ssh_key_path} {user_name}@{public_ip} '{cmd}'",
            ssh_key_path = self.ssh_key_path,
            user_name = self.user_name,
            public_ip = self.public_ip,
        );
        command_manager::run(&remote_cmd_to_run)
    }

    pub fn ssm_start_session_command(&self) -> String {
        // aws ssm start-session --region [region] --target [instance ID]
        format!(
            "aws ssm start-session --region {region} --target {instance_id}",
            region = self.region,
            instance_id = self.instance_id,
        )
    }

    /// Downloads a remote file to the local machine.
    pub fn download_file(
        &self,
        remote_file_path: &str,
        local_file_path: &str,
        overwrite: bool,
    ) -> io::Result<command_manager::Output> {
        log::info!("sending an SCP command to {}", self.public_ip);
        if Path::new(local_file_path).exists() && !overwrite {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("file '{local_file_path}' already exists"),
            ));
        }
        if overwrite {
            let local_rm_cmd = format!("rm -f {local_file_path} || true");
            let rm_out = command_manager::run(&local_rm_cmd)?;
            log::info!("successfully rm '{local_file_path}' (out {:?})", rm_out);
        };

        let remote_cmd_to_run = format!("chmod 400 {ssh_key_path} && scp -i {ssh_key_path} {user_name}@{public_ip}:{remote_file_path} {local_file_path}",
            ssh_key_path = self.ssh_key_path,
            user_name = self.user_name,
            public_ip = self.public_ip,
            remote_file_path = remote_file_path,
            local_file_path = local_file_path,
        );
        let out = command_manager::run(&remote_cmd_to_run)?;

        if Path::new(local_file_path).exists() {
            log::info!("successfully downloaded to '{local_file_path}'")
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("file '{local_file_path}' does not exist"),
            ));
        }

        Ok(out)
    }

    /// Sends a local file to the remote machine.
    pub fn send_file(
        &self,
        local_file_path: &str,
        remote_file_path: &str,
        overwrite: bool,
    ) -> io::Result<command_manager::Output> {
        log::info!("send_file to {}", self.public_ip);
        if !Path::new(local_file_path).exists() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("file '{local_file_path}' does not exist"),
            ));
        }

        if overwrite {
            let remote_rm_cmd = format!("chmod 400 {ssh_key_path} && ssh -o \"StrictHostKeyChecking no\" -i {ssh_key_path} {user_name}@{public_ip} 'sudo rm -f {remote_file_path} || true'",
                ssh_key_path = self.ssh_key_path,
                user_name = self.user_name,
                public_ip = self.public_ip,
            );
            let rm_out = command_manager::run(&remote_rm_cmd)?;
            log::info!("successfully rm '{remote_file_path}' (out {:?})", rm_out);
        };

        let remote_cmd_to_run = format!("chmod 400 {ssh_key_path} && scp -i {ssh_key_path} {local_file_path} {user_name}@{public_ip}:{remote_file_path}",
            ssh_key_path = self.ssh_key_path,
            user_name = self.user_name,
            public_ip = self.public_ip,
            local_file_path = local_file_path,
            remote_file_path = remote_file_path,
        );
        let out = command_manager::run(&remote_cmd_to_run)?;

        let remote_ls_cmd = format!("chmod 400 {ssh_key_path} && ssh -o \"StrictHostKeyChecking no\" -i {ssh_key_path} {user_name}@{public_ip} 'ls {remote_file_path}'",
            ssh_key_path = self.ssh_key_path,
            user_name = self.user_name,
            public_ip = self.public_ip,
        );
        let ls_out = command_manager::run(&remote_ls_cmd)?;
        log::info!(
            "successfully sent to '{remote_file_path}' (out {:?})",
            ls_out
        );

        Ok(out)
    }

    /// Downloads a remote directory to the local machine.
    pub fn download_directory(
        &self,
        remote_directory_path: &str,
        local_directory_path: &str,
        overwrite: bool,
    ) -> io::Result<command_manager::Output> {
        log::info!("download_directory from {}", self.public_ip);
        if Path::new(local_directory_path).exists() && !overwrite {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("directory '{local_directory_path}' already exists"),
            ));
        }
        if overwrite {
            let local_rm_cmd = format!("rm -rf {local_directory_path} || true");
            let rm_out = command_manager::run(&local_rm_cmd)?;
            log::info!(
                "successfully rm '{local_directory_path}' (out {:?})",
                rm_out
            );
        };

        let remote_cmd_to_run = format!("chmod 400 {ssh_key_path} && scp -i {ssh_key_path} -r {user_name}@{public_ip}:{remote_directory_path} {local_directory_path}",
            ssh_key_path = self.ssh_key_path,
            user_name = self.user_name,
            public_ip = self.public_ip,
            remote_directory_path = remote_directory_path,
            local_directory_path = local_directory_path,
        );
        let out = command_manager::run(&remote_cmd_to_run)?;

        if Path::new(local_directory_path).exists() {
            log::info!("successfully downloaded to '{local_directory_path}'")
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("directory '{local_directory_path}' does not exist"),
            ));
        }

        Ok(out)
    }

    /// Sends a local directory to the remote machine.
    pub fn send_directory(
        &self,
        local_directory_path: &str,
        remote_directory_path: &str,
        overwrite: bool,
    ) -> io::Result<command_manager::Output> {
        log::info!("send_directory to {}", self.public_ip);
        if !Path::new(local_directory_path).exists() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("file '{local_directory_path}' does not exist"),
            ));
        }

        if overwrite {
            let remote_rm_cmd = format!("chmod 400 {ssh_key_path} && ssh -o \"StrictHostKeyChecking no\" -i {ssh_key_path} {user_name}@{public_ip} 'sudo rm -f {remote_directory_path} || true'",
                ssh_key_path = self.ssh_key_path,
                user_name = self.user_name,
                public_ip = self.public_ip,
            );
            let rm_out = command_manager::run(&remote_rm_cmd)?;
            log::info!(
                "successfully rm '{remote_directory_path}' (out {:?})",
                rm_out
            );
        };

        let remote_cmd_to_run = format!("chmod 400 {ssh_key_path} && scp -i {ssh_key_path} -r {local_directory_path} {user_name}@{public_ip}:{remote_directory_path}",
            ssh_key_path = self.ssh_key_path,
            user_name = self.user_name,
            public_ip = self.public_ip,
            local_directory_path = local_directory_path,
            remote_directory_path = remote_directory_path,
        );
        let out = command_manager::run(&remote_cmd_to_run)?;

        let remote_ls_cmd = format!("chmod 400 {ssh_key_path} && ssh -o \"StrictHostKeyChecking no\" -i {ssh_key_path} {user_name}@{public_ip} 'ls {remote_directory_path}'",
            ssh_key_path = self.ssh_key_path,
            user_name = self.user_name,
            public_ip = self.public_ip,
        );
        let ls_out = command_manager::run(&remote_ls_cmd)?;
        log::info!(
            "successfully sent to '{remote_directory_path}' (out {:?})",
            ls_out
        );

        Ok(out)
    }
}

/// A list of ssh commands.
pub struct Commands(pub Vec<Command>);

impl Commands {
    pub fn sync(&self, file_path: &str) -> io::Result<()> {
        log::info!("syncing ssh commands to '{file_path}'");
        let path = Path::new(file_path);
        let parent_dir = path.parent().unwrap();
        fs::create_dir_all(parent_dir)?;

        let mut contents = String::from("#!/bin/bash\n\n");
        for ssh_cmd in self.0.iter() {
            let d = ssh_cmd.to_string();
            contents.push_str(&d);
            contents.push_str("\n\n");
        }

        let mut f = File::create(file_path)?;
        f.write_all(&contents.as_bytes())?;

        Ok(())
    }
}
