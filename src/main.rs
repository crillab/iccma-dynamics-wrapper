// iccma21-dynamics-wrapper
// Copyright (C) 2020  Artois University and CNRS
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//
// Contributors:
//   *   CRIL - initial API and implementation

mod app;

use app::wrap_command::WrapCommand;
use crusti_app_helper::{AppHelper, Command, LicenseCommand};

fn main() {
    let mut app = AppHelper::new(
        option_env!("CARGO_PKG_NAME").unwrap_or("unknown app name"),
        option_env!("CARGO_PKG_VERSION").unwrap_or("unknown version"),
        "Emmanuel Lonca <lonca@cril.fr>",
        "An app for Feature Models.",
    );
    let commands: Vec<Box<dyn Command>> = vec![
        Box::new(WrapCommand::new()),
        Box::new(LicenseCommand::new(include_str!("../LICENSE").to_string())),
    ];
    for c in commands {
        app.add_command(c);
    }
    app.launch_app();
}
