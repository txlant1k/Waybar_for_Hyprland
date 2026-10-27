use std::fs::File;
use std::io::{BufRead, BufReader};
/// Читает первую строку `/proc/stat` и возвращает счётчики процессора.
///
/// Возвращает пару `(всего, простой)`, где «простой» — это сумма полей
/// `idle` и `iowait`. Счётчики растут с момента загрузки системы, поэтому
/// загрузку нужно считать по разнице двух замеров.
///
/// # Errors
///
/// Возвращает ошибку, если файл `/proc/stat` не удалось открыть или прочитать
pub fn read_cpu() -> std::io::Result<(u64, u64)> {
    let file = File::open("/proc/stat")?;
    let reader = BufReader::new(file);
    let proc_stat: Vec<u64>; // вектор для хранения счетчиков процессора
    if let Some(first_line) = reader.lines().next() {
        let line = first_line?;
        proc_stat = line.split_whitespace().skip(1).map(|s| s.parse().unwrap()).collect();
    } else {
        return Ok((0, 0));
    }
    let simple = proc_stat[3] + proc_stat[4];
    let all = proc_stat.into_iter().sum();
    Ok((all, simple))
}/// Читает `/proc/meminfo` и возвращает занятую и общую память в гигабайтах.
///
/// Занятая память считается как `MemTotal - MemAvailable`.
///
/// # Errors
///
/// Возвращает ошибку, если файл не удалось открыть или прочитать.
pub fn read_memory() -> std::io::Result<(f64, f64)> {
    let target_word1 = "MemTotal";
    let mut memory_total: u64 = 0;
    let target_word2 = "MemAvailable";
    let mut memory_available: u64 = 0;
    // Открываем файл и создаем объект BufReader для построчного чтения
    let file = File::open("/proc/meminfo")?;
    let reader = BufReader::new(file);
    // Цикл проходит весь файл и ищет строки, наинающиеся с заданных слов target_word1 
    // и target_word2 соответственно
    for line in reader.lines() {
        // проверка на возможность прочитать строку, в противном случае возвращаем ошибку
        let line = line?;
        if line.starts_with(target_word1) {
            if let Ok(value) = line.split_whitespace().nth(1).unwrap().parse() {
                memory_total = value;
            }
        }
        if line.starts_with(target_word2) {
            if let Ok(value) = line.split_whitespace().nth(1).unwrap().parse() {
                memory_available = value;
            }
        }
    }
    // находим количество занятой памяти и переводим его в гигабайты вместе с общим количеством оной
    let occupied_memory: f64 = ((memory_total - memory_available) as f64) / (1024.0 * 1024.0);
    let memory_total_f64 = memory_total as f64 / (1024.0 * 1024.0);
    // возвращаем пару значений (используется, всего)
    Ok((occupied_memory, memory_total_f64))
}
