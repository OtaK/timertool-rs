use winapi::{shared::wtypes::BSTR, um::taskschd::IRegistrationInfo};

use super::IUnknownWrapper;

pub struct TaskRegistrationInfo(IUnknownWrapper<IRegistrationInfo>);

impl From<*mut IRegistrationInfo> for TaskRegistrationInfo {
    fn from(registration_info: *mut IRegistrationInfo) -> Self {
        Self(registration_info.into())
    }
}

impl Into<*mut IRegistrationInfo> for TaskRegistrationInfo {
    fn into(self) -> *mut IRegistrationInfo {
        self.0.into()
    }
}

#[allow(dead_code)]
impl TaskRegistrationInfo {
    pub fn description(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_Description(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_description<S: AsRef<str>>(&self, description: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_Description(crate::wstr!(description.as_ref())))?;
        Ok(())
    }

    pub fn author(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_Author(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_author<S: AsRef<str>>(&self, author: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_Author(crate::wstr!(author.as_ref())))?;
        Ok(())
    }

    pub fn version(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_Version(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_version<S: AsRef<str>>(&self, version: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_Version(crate::wstr!(version.as_ref())))?;
        Ok(())
    }

    pub fn date(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_Date(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_date<S: AsRef<str>>(&self, date: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_Date(crate::wstr!(date.as_ref())))?;
        Ok(())
    }

    pub fn documentation(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_Documentation(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_documentation<S: AsRef<str>>(&self, documentation: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_Documentation(crate::wstr!(documentation.as_ref())))?;
        Ok(())
    }

    pub fn xml_text(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_XmlText(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_xml_text<S: AsRef<str>>(&self, xml_text: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_XmlText(crate::wstr!(xml_text.as_ref())))?;
        Ok(())
    }

    pub fn uri(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_URI(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_uri<S: AsRef<str>>(&self, uri: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_URI(crate::wstr!(uri.as_ref())))?;
        Ok(())
    }

    pub fn source(&self) -> std::io::Result<String> {
        let mut ret: BSTR = std::ptr::null_mut();
        crate::w32_ok!((*self.0).get_Source(&mut ret))?;
        super::bstr_to_string(ret)
    }

    pub fn set_source<S: AsRef<str>>(&self, source: S) -> std::io::Result<()> {
        crate::w32_ok!((*self.0).put_Source(crate::wstr!(source.as_ref())))?;
        Ok(())
    }
}
