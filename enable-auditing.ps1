if (-not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator"))
{
    Start-Process powershell -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`"" -Verb RunAs
    exit
}
$auditCategories = @(
    "Audit Process Creation",
    "Audit Process Termination"
)

foreach ($category in $auditCategories) {
    auditpol /set /subcategory:"$category" /success:enable /failure:enable
}
$regPath = "HKLM:\Software\Microsoft\Windows\CurrentVersion\Policies\System\Audit"
$regName = "ProcessCreationIncludeCmdLine_Enabled"
If (!(Test-Path $regPath)) {
    New-Item -Path $regPath -Force | Out-Null
}
Set-ItemProperty -Path $regPath -Name $regName -Value 1
Write-Output "Process start and terminate auditing enabled successfully."
