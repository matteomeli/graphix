use std::ptr;

use winapi::shared::winerror;
use winapi::um::{d3d12, handleapi, synchapi, winnt};
use wio::com::ComPtr;

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Event {
    handle: winnt::HANDLE,
}

impl Event {
    pub fn new(manual_reset: bool, initial_state: bool) -> Self {
        Event {
            handle: unsafe {
                synchapi::CreateEventA(
                    ptr::null_mut(),
                    manual_reset as _,
                    initial_state as _,
                    ptr::null(),
                )
            },
        }
    }

    pub fn wait(self, timeout_ms: u32) -> u32 {
        unsafe { synchapi::WaitForSingleObject(self.handle, timeout_ms) }
    }

    pub fn close(self) {
        unsafe { handleapi::CloseHandle(self.handle) };
    }
}

pub struct Fence {
    pub(crate) raw: ComPtr<d3d12::ID3D12Fence>,
}

impl Fence {
    pub fn as_ptr(&self) -> *const d3d12::ID3D12Fence {
        self.raw.as_raw()
    }

    pub fn as_mut_ptr(&self) -> *mut d3d12::ID3D12Fence {
        self.raw.as_raw()
    }

    pub fn signal(&self, value: u64) -> winerror::HRESULT {
        unsafe { self.raw.Signal(value) }
    }

    pub fn get_value(&self) -> u64 {
        unsafe { self.raw.GetCompletedValue() }
    }

    pub fn set_event_on_completion(&self, event: Event, value: u64) -> winerror::HRESULT {
        unsafe { self.raw.SetEventOnCompletion(value, event.handle) }
    }

    pub fn wait_for_value(&self, event: Event, value: u64) {
        self.wait_for_value_timeout(event, value, u32::max_value());
    }

    pub fn wait_for_value_timeout(&self, event: Event, value: u64, timeout_ms: u32) {
        if self.get_value() >= value {
            return;
        }

        let hr = self.set_event_on_completion(event, value);
        if !winerror::SUCCEEDED(hr) {
            panic!("Failed on setting fence event on completion: {:?}", hr);
        }

        event.wait(timeout_ms);
    }
}
