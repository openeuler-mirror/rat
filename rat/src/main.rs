// This file is part of the rat package.
//
// (c) Junbin Zhang <1127626033@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use rat::{parse_cmd_args, rat_process};
use std::process;

fn main() {
    let config = parse_cmd_args().expect("Error in parse args");
    process::exit(rat_process(&config));
}
