// Copyright (C) 2024 Neuversity
// All Rights Reserved.
//
// NOTICE: All information contained herein is, and remains
// the property of Neuversity.
// The intellectual and technical concepts contained
// herein are proprietary to Neuversity
// and are protected by trade secret or copyright law.
// Dissemination of this information or reproduction of this material
// is strictly forbidden unless prior written permission is obtained
// from Neuversity.

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub listen: String, // 127.0.0.1:8080
}
