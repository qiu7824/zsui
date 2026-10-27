param(
    [Parameter(Mandatory)][string]$Executable,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [ValidateSet('baseline','candidate')][string]$Variant,
    [int]$Trial,
    [ValidateRange(200,1000)][int]$Operations=200,
    [switch]$VerifyEmptyReplacement
)
$ErrorActionPreference='Stop'
$taskRoot=[IO.Path]::GetFullPath($OutputDirectory)
$artifact=(Resolve-Path -LiteralPath $Executable).Path
$build=[pscustomobject]@{artifact=$artifact;working_directory=(Split-Path -Parent $artifact);sha256=(Get-FileHash -LiteralPath $artifact -Algorithm SHA256).Hash}
$output=Join-Path $taskRoot "$Variant-$Trial"
if(Test-Path -LiteralPath $output){throw 'Latency trial already exists'}
New-Item -ItemType Directory -Path $output | Out-Null
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
if(-not ('ZsuiLatencyNative' -as [type])) {
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class ZsuiLatencyNative {
  private delegate bool EnumProc(IntPtr hwnd, IntPtr data);
  [DllImport("user32.dll")] private static extern bool EnumWindows(EnumProc callback, IntPtr data);
  [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] private static extern int GetClassName(IntPtr hwnd, StringBuilder name, int length);
  [DllImport("user32.dll")] private static extern IntPtr SendMessageW(IntPtr hwnd, uint msg, UIntPtr wparam, IntPtr lparam);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hwnd, int show);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
  [DllImport("user32.dll")] private static extern bool RedrawWindow(IntPtr hwnd, IntPtr rect, IntPtr region, uint flags);
  [DllImport("dwmapi.dll")] private static extern int DwmFlush();
  [DllImport("user32.dll")] private static extern bool ScreenToClient(IntPtr hwnd, ref Point point);
  [StructLayout(LayoutKind.Sequential)] private struct Point { public int X,Y; }
  public static IntPtr Find(uint pid) {
    IntPtr found=IntPtr.Zero;
    EnumWindows((hwnd,data)=> { uint owner; GetWindowThreadProcessId(hwnd,out owner);
      if(owner==pid){var name=new StringBuilder(128);GetClassName(hwnd,name,128);if(name.ToString()=="ZsuiMainWindow"){found=hwnd;return false;}}return true;
    },IntPtr.Zero); return found;
  }
  public static void Click(IntPtr hwnd,int screenX,int screenY) {
    var point=new Point{X=screenX,Y=screenY}; ScreenToClient(hwnd,ref point);
    var packed=new IntPtr((point.Y<<16)|(point.X&0xffff));
    SendMessageW(hwnd,0x0201,new UIntPtr(1),packed); SendMessageW(hwnd,0x0202,UIntPtr.Zero,packed);
  }
  public static void Character(IntPtr hwnd,int character) {SendMessageW(hwnd,0x0102,new UIntPtr((uint)character),IntPtr.Zero);}
  public static void Wheel(IntPtr hwnd,int delta,int screenX,int screenY) {
    SendMessageW(hwnd,0x020A,new UIntPtr(unchecked((uint)(delta<<16))),new IntPtr((screenY<<16)|(screenX&0xffff)));
  }
  public static void Frame(IntPtr hwnd) {
    if(!RedrawWindow(hwnd,IntPtr.Zero,IntPtr.Zero,0x181))throw new InvalidOperationException("Native repaint failed");
    int result=DwmFlush(); if(result<0)Marshal.ThrowExceptionForHR(result);
  }
  public static void Close(IntPtr hwnd){SendMessageW(hwnd,0x0010,UIntPtr.Zero,IntPtr.Zero);}
}
'@
}
$process=Start-Process -FilePath $build.artifact -WorkingDirectory $build.working_directory -WindowStyle Hidden -PassThru
$birth=$process.StartTime.ToFileTimeUtc()
$record=[ordered]@{variant=$Variant;trial=$Trial;operations=$Operations;binary_sha256=$build.sha256;pid=$process.Id;creation_ticks=$birth;status='running';owner='zsui-interaction-latency';output=$output;measurement_version=2}
$record | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $taskRoot 'latency-process.json') -Encoding utf8
try {
  $hwnd=[IntPtr]::Zero
  for($i=0;$i -lt 200 -and $hwnd -eq [IntPtr]::Zero;$i++) {if($process.HasExited){throw 'Latency app exited'};$hwnd=[ZsuiLatencyNative]::Find([uint32]$process.Id);Start-Sleep -Milliseconds 50}
  if($hwnd -eq [IntPtr]::Zero){throw 'No native window'}
  [void][ZsuiLatencyNative]::ShowWindow($hwnd,5)
  [void][ZsuiLatencyNative]::SetForegroundWindow($hwnd)
  $activator=New-Object -ComObject WScript.Shell
  try {$record.activation_requested=$activator.AppActivate($process.Id)}
  finally {[void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($activator)}
  Start-Sleep -Seconds 3
  if([ZsuiLatencyNative]::GetForegroundWindow() -ne $hwnd){throw 'Measurement target did not acquire foreground'}
  $root=[System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
  $condition=[System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::ControlTypeProperty,[System.Windows.Automation.ControlType]::Edit)
  $editor=$root.FindFirst([System.Windows.Automation.TreeScope]::Descendants,$condition)
  if($null -eq $editor){throw 'No native accessible editor'}
  $rect=$editor.Current.BoundingRectangle
  $x=[int]($rect.Left+$rect.Width/2);$y=[int]($rect.Top+$rect.Height/2)
  [ZsuiLatencyNative]::Click($hwnd,$x,$y)
  $value=$editor.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
  $record.editor=@{id=$editor.Current.AutomationId;class=$editor.Current.ClassName;rectangle=$rect.ToString()}
  if($VerifyEmptyReplacement){
    $value.SetValue('')
    for($i=0;$i -lt 100 -and $value.Current.Value -ne '';$i++){[ZsuiLatencyNative]::Frame($hwnd);Start-Sleep -Milliseconds 20}
    if($value.Current.Value -ne ''){throw 'Empty UIA replacement regression'}
    $record.empty_uia_replacement_verified=$true
  }
  # Both variants use the same nonempty replacement plus native Backspace;
  # the baseline's separately recorded empty-SetValue bug must not bias timing.
  $value.SetValue('X')
  for($i=0;$i -lt 100 -and $value.Current.Value -ne 'X';$i++){[ZsuiLatencyNative]::Frame($hwnd);Start-Sleep -Milliseconds 20}
  if($value.Current.Value -ne 'X'){$record.unexpected_value=$value.Current.Value;throw 'UIA replacement did not reach application state'}
  [ZsuiLatencyNative]::Character($hwnd,8);[ZsuiLatencyNative]::Frame($hwnd)
  if($value.Current.Value -ne ''){throw 'Native Backspace did not clear the fixture'}
  $inputTimes=[Collections.Generic.List[double]]::new()
  $scrollTimes=[Collections.Generic.List[double]]::new()
  $restoreTimes=[Collections.Generic.List[double]]::new()
  $timer=[Diagnostics.Stopwatch]::new()
  for($i=0;$i -lt $Operations;$i++) {
    if([ZsuiLatencyNative]::GetForegroundWindow() -ne $hwnd -or -not $editor.Current.HasKeyboardFocus){
      $record.focus_lost_before_input=$i
      throw 'Measurement target lost foreground or editor focus'
    }
    $timer.Restart();[ZsuiLatencyNative]::Character($hwnd,0x4e2d);[ZsuiLatencyNative]::Frame($hwnd);$timer.Stop();$inputTimes.Add($timer.Elapsed.TotalMilliseconds)
  }
  if($value.Current.Value -ne ('中'*$Operations)){
    $record.input_value_at_check=$value.Current.Value
    $record.input_value_length_at_check=$value.Current.Value.Length
    $record.input_times_before_failure=$inputTimes.ToArray()
    for($i=0;$i -lt 100 -and $value.Current.Value -ne ('中'*$Operations);$i++){
      [ZsuiLatencyNative]::Frame($hwnd);Start-Sleep -Milliseconds 20
    }
    $record.input_value_after_settling=$value.Current.Value
    $record.input_value_length_after_settling=$value.Current.Value.Length
    $record.foreground_at_failure=[ZsuiLatencyNative]::GetForegroundWindow().ToInt64()
    $record.target_hwnd=$hwnd.ToInt64()
    $record.editor_has_focus_at_failure=$editor.Current.HasKeyboardFocus
    [ZsuiLatencyNative]::Click($hwnd,$x,$y);[ZsuiLatencyNative]::Frame($hwnd)
    $freshEditor=$root.FindFirst([System.Windows.Automation.TreeScope]::Descendants,$condition)
    $record.input_value_after_refocus=$freshEditor.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value
    throw 'Native input completion was not established at the measurement boundary'
  }
  $content=(0..199 | ForEach-Object {"Line $_ / 中文 العربية"}) -join "`n"
  $value.SetValue($content);[ZsuiLatencyNative]::Frame($hwnd)
  $text=$editor.GetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern)
  $range=$text.DocumentRange.Clone()
  $range.MoveEndpointByRange([System.Windows.Automation.Text.TextPatternRangeEndpoint]::End,$text.DocumentRange,[System.Windows.Automation.Text.TextPatternRangeEndpoint]::Start)
  [void]$range.MoveEndpointByUnit([System.Windows.Automation.Text.TextPatternRangeEndpoint]::End,[System.Windows.Automation.Text.TextUnit]::Character,1)
  $range.ScrollIntoView($true);[ZsuiLatencyNative]::Frame($hwnd)
  $before=$range.GetBoundingRectangles() -join ','
  [ZsuiLatencyNative]::Wheel($hwnd,-120,$x,$y);[ZsuiLatencyNative]::Frame($hwnd)
  $after=$range.GetBoundingRectangles() -join ','
  if($before -eq $after){throw 'Scroll did not change visible text geometry'}
  $range.ScrollIntoView($true);[ZsuiLatencyNative]::Frame($hwnd)
  for($i=0;$i -lt $Operations;$i++) {
    $delta=if($i%2){120}else{-120}
    $timer.Restart();[ZsuiLatencyNative]::Wheel($hwnd,$delta,$x,$y);[ZsuiLatencyNative]::Frame($hwnd);$timer.Stop();$scrollTimes.Add($timer.Elapsed.TotalMilliseconds)
  }
  for($i=0;$i -lt $Operations;$i++) {
    [void][ZsuiLatencyNative]::ShowWindow($hwnd,0)
    $timer.Restart();[void][ZsuiLatencyNative]::ShowWindow($hwnd,4);[ZsuiLatencyNative]::Frame($hwnd);$timer.Stop();$restoreTimes.Add($timer.Elapsed.TotalMilliseconds)
    if(-not [ZsuiLatencyNative]::IsWindowVisible($hwnd)){throw 'Restore left the window hidden'}
  }
  [ZsuiLatencyNative]::Click($hwnd,$x,$y);[ZsuiLatencyNative]::Frame($hwnd)
  $editor=$root.FindFirst([System.Windows.Automation.TreeScope]::Descendants,$condition)
  $value=$editor.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
  if($value.Current.Value -ne $content){throw 'Text state changed during hide/restore'}
  $record.input_ms=$inputTimes.ToArray();$record.scroll_ms=$scrollTimes.ToArray();$record.restore_ms=$restoreTimes.ToArray()
  $record.scroll_geometry_changed=$true;$record.text_preserved=$true;$record.status='passed'
} catch {$record.status='failed';$record.error=$_.ToString();throw}
finally {
  if(-not $process.HasExited){
    $current=Get-Process -Id $process.Id -ErrorAction SilentlyContinue
    if($current -and $current.StartTime.ToFileTimeUtc() -eq $birth -and $current.Path -eq $build.artifact){
      if($hwnd -ne [IntPtr]::Zero){
        [ZsuiLatencyNative]::Close($hwnd)
        if(-not $process.WaitForExit(500)){
          try {
            $root=[System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
            $discardCondition=[System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::NameProperty,'放弃 / Discard')
            $discard=$root.FindFirst([System.Windows.Automation.TreeScope]::Descendants,$discardCondition)
            if($discard){$discard.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()}
          } catch {}
        }
      }
      if(-not $process.WaitForExit(3000)){$current.Kill();$current.WaitForExit();$record.cleanup='owned synthetic document process stopped'}
      else {$record.cleanup='synthetic changes discarded; native process exited'}
    }
  }
  $record.finished_at=(Get-Date).ToUniversalTime().ToString('o')
  $record | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $output 'latency.json') -Encoding utf8
  $process.Dispose()
}
Write-Output "$Variant latency trial $Trial passed: $Operations input, scroll and restore operations"
