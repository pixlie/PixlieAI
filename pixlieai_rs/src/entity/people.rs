// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

pub struct Person {
    pub id: u32,

    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

pub struct PhoneNumber {
    pub country_code: String,
    pub number: String,
}

pub struct Address {
    pub street: String,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: String,
    pub country: String,
}

pub enum ContactType {
    Email,
    Phone(PhoneNumber),
    WhatsApp(PhoneNumber),
    Telegram(PhoneNumber),
    AddressSigle(String),
    Address(Address),
    Twitter,
    Facebook,
    Instagram,
    LinkedIn,
    GitHub,
    Other(String),
}

pub struct Contact {
    pub contact_type: ContactType,
    pub value: String,
}
