param(
    [string]$Source = "$PSScriptRoot\..\artifacts\session.bmp",
    [string]$Destination = "$PSScriptRoot\..\artifacts\session.png"
)

# The diagnostic BMP stores premultiplied alpha. Preserve it when exporting PNG.
Add-Type -AssemblyName System.Drawing
$surfaceBytes = [System.IO.File]::ReadAllBytes($Source)
$surfaceWidth = [BitConverter]::ToInt32($surfaceBytes, 18)
$surfaceHeight = -[BitConverter]::ToInt32($surfaceBytes, 22)
$surface = [System.Drawing.Bitmap]::new($surfaceWidth, $surfaceHeight, [System.Drawing.Imaging.PixelFormat]::Format32bppPArgb)
try {
    $bounds = [System.Drawing.Rectangle]::new(0, 0, $surfaceWidth, $surfaceHeight)
    $locked = $surface.LockBits($bounds, [System.Drawing.Imaging.ImageLockMode]::WriteOnly, [System.Drawing.Imaging.PixelFormat]::Format32bppPArgb)
    try {
        [System.Runtime.InteropServices.Marshal]::Copy($surfaceBytes, 54, $locked.Scan0, $surfaceWidth * $surfaceHeight * 4)
    } finally { $surface.UnlockBits($locked) }
    $surface.Save($Destination, [System.Drawing.Imaging.ImageFormat]::Png)
} finally { $surface.Dispose() }
