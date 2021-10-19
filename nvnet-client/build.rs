fn main() {
    windows::build! {
        Windows::Win32::Storage::FileSystem::CreateFileW,
        Windows::Win32::System::SystemServices::DeviceIoControl,
    };
}
