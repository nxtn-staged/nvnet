fn main() {
    windows::build!(
        windows::win32::debug::{GetLastError, RtlNtStatusToDosError},
        windows::win32::file_system::CreateFileW,
        windows::win32::system_services::DeviceIoControl,
        windows::win32::windows_programming::CloseHandle,
    );
}
