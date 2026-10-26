param(
    [int]$Width = 680,
    [int]$Height = 560,
    [int]$IntervalMs = 30
)

# Negative control for scripts/measure-proxy-demo-comparison.ps1: a Windows
# Forms window that erases its background and then draws rows one at a time
# without double buffering. The flicker probe must detect this window, which
# shows that a clean result for ZSUI is a measurement, not a blind spot.

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$form = New-Object System.Windows.Forms.Form
$form.Text = "ZSUI flicker control"
$form.ClientSize = New-Object System.Drawing.Size($Width, $Height)
$form.StartPosition = "Manual"
$form.Location = New-Object System.Drawing.Point(0, 0)
$form.TopMost = $true
$form.BackColor = [System.Drawing.Color]::FromArgb(243, 243, 243)

$rowBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
$textBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(26, 26, 26))
$form.Add_Paint({
    param($sender, $event)
    $graphics = $event.Graphics
    $rows = [Math]::Floor(($sender.ClientSize.Height - 40) / 24)
    for ($row = 0; $row -lt $rows; $row++) {
        $top = 20 + $row * 24
        $graphics.FillRectangle($rowBrush, 20, $top, $sender.ClientSize.Width - 40, 20)
        $graphics.DrawString("unbuffered row $row", $sender.Font, $textBrush, 26, $top + 3)
        [System.Threading.Thread]::Sleep(1)
    }
})

$timer = New-Object System.Windows.Forms.Timer
$timer.Interval = $IntervalMs
$timer.Add_Tick({ $form.Invalidate() })
$timer.Start()
[System.Windows.Forms.Application]::Run($form)
