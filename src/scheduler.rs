use std::fs::File;
use std::io::Write;
use std::process::Command;

pub fn create_scheduled_task(task_name: &str, device_path: &str) -> std::io::Result<()> {
    let task_template = r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.4" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Author>TinyTimeTracker</Author>
    <URI>{TaskNameTemp}</URI>
  </RegistrationInfo>
  <Triggers>
    <EventTrigger>
      <Enabled>true</Enabled>
      <Subscription>&lt;QueryList&gt;&lt;Query Id="0" Path="Security"&gt;&lt;Select Path="Security"&gt;
      *[System[Provider[@Name='Microsoft-Windows-Security-Auditing'] and (EventID=4688)]]
      and
      *[EventData[Data[@Name='NewProcessName'] and (Data='{FilePathTemp}')]]
    &lt;/Select&gt;&lt;/Query&gt;&lt;/QueryList&gt;</Subscription>
    </EventTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <LogonType>InteractiveToken</LogonType>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>true</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>true</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>false</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <IdleSettings>
      <StopOnIdleEnd>true</StopOnIdleEnd>
      <RestartOnIdle>false</RestartOnIdle>
    </IdleSettings>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>false</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <DisallowStartOnRemoteAppSession>false</DisallowStartOnRemoteAppSession>
    <UseUnifiedSchedulingEngine>true</UseUnifiedSchedulingEngine>
    <WakeToRun>false</WakeToRun>
    <ExecutionTimeLimit>PT72H</ExecutionTimeLimit>
    <Priority>7</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>powershell.exe</Command>
    </Exec>
  </Actions>
</Task>
"#;

    let task_content = task_template
    .replace("{TaskNameTemp}", task_name)
    .replace("{FilePathTemp}", device_path);

    let temp_xml_path = "temp-task.xml";
    {
        let mut file = File::create(temp_xml_path)?;
        file.write_all(task_content.as_bytes())?;
        file.flush()?;
    }
    println!("Written XML:\n{}", task_content);
    let output = Command::new("schtasks")
        .args(&[
            "/Create",
            "/TN",
            task_name,
            "/XML",
            temp_xml_path,
            "/F",
        ])
        .output()?;
    if output.status.success() {
        println!("Task '{}' created successfully.", task_name);
    } else {
        std::fs::remove_file(temp_xml_path)?;
        eprintln!(
            "Failed to create task '{}': {}",
            task_name,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::remove_file(temp_xml_path)?;
    Ok(())
}
