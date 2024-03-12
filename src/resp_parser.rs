use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::tcp::OwnedReadHalf,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RESPType {
    SimpleString(String),
    BulkString(String),
    Array(Vec<RESPType>),
    NullBulkString,
}

impl RESPType {
    pub fn serialize(&self) -> String {
        match self {
            RESPType::SimpleString(s) => format!("+{}\r\n", s),
            RESPType::BulkString(s) => format!("${}\r\n{}\r\n", s.len(), s),
            RESPType::Array(a) => {
                let mut result = format!("*{}\r\n", a.len());
                for item in a {
                    result.push_str(&item.serialize());
                }
                result
            }
            RESPType::NullBulkString => "$-1\r\n".to_string(),
        }
    }
}

pub struct RESPParser {
    reader: BufReader<OwnedReadHalf>,
}

impl RESPParser {
    pub fn new(stream: BufReader<OwnedReadHalf>) -> Self {
        RESPParser { reader: stream }
    }

    pub async fn parse(&mut self) -> Result<Option<RESPType>, anyhow::Error> {
        let line = RESPParser::read_line(&mut self.reader).await?;

        match line.chars().next() {
            Some('+') => Ok(Some(RESPType::SimpleString(
                Self::read_line(&mut self.reader).await?,
            ))),
            Some('$') => Self::read_bulk_string(&mut self.reader).await,
            Some('*') => Self::read_array(self, line).await,
            _ => Ok(None),
        }
    }

    async fn read_line(stream: &mut BufReader<OwnedReadHalf>) -> anyhow::Result<String> {
        let mut line = String::new();
        stream.read_line(&mut line).await?;
        let result = line.trim().to_owned();
        Ok(result)
    }

    async fn read_bulk_string(
        stream: &mut BufReader<OwnedReadHalf>,
    ) -> Result<Option<RESPType>, anyhow::Error> {
        let data = RESPParser::read_line(stream).await?;
        Ok(Some(RESPType::BulkString(data)))
    }

    #[async_recursion]
    async fn read_array(&mut self, line: String) -> Result<Option<RESPType>, anyhow::Error> {
        let array_size: usize = line.chars().skip(1).collect::<String>().parse()?;
        let mut output = Vec::with_capacity(array_size);
        for _ in 0..array_size {
            let item = self.parse().await?.ok_or(anyhow!(
                "Expected {} elements but some were missing",
                array_size
            ))?;
            output.push(item);
        }
        Ok(Some(RESPType::Array(output)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    use tokio::net::{TcpListener, TcpStream};

    async fn setup_test_server(resp: &'static str) -> BufReader<OwnedReadHalf> {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server_task = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            socket.write_all(resp.as_bytes()).await.unwrap();
        });

        let stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        server_task.await.unwrap();
        BufReader::new(stream.into_split().0)
    }

    #[tokio::test]
    async fn test_parse_bulk_string() {
        let stream = setup_test_server("$5\r\nhello\r\n").await;
        let mut parser = RESPParser::new(stream);
        let resp = parser.parse().await.unwrap().unwrap();
        assert_eq!(resp, RESPType::BulkString("hello".to_string()));
    }

    #[tokio::test]
    async fn test_parse_array() {
        let stream = setup_test_server("*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n").await;
        let mut parser = RESPParser::new(stream);
        let resp = parser.parse().await.unwrap().unwrap();
        assert_eq!(
            resp,
            RESPType::Array(vec![
                RESPType::BulkString("hello".to_string()),
                RESPType::BulkString("world".to_string()),
            ])
        );
    }

    #[tokio::test]
    async fn test_serialize_simple_string() {
        let resp = RESPType::SimpleString("hello".to_string());
        assert_eq!(resp.serialize(), "+hello\r\n");
    }

    #[tokio::test]
    async fn test_serialize_bulk_string() {
        let resp = RESPType::BulkString("hello".to_string());
        assert_eq!(resp.serialize(), "$5\r\nhello\r\n");
    }

    #[tokio::test]
    async fn test_serialize_array() {
        let resp = RESPType::Array(vec![
            RESPType::BulkString("hello".to_string()),
            RESPType::BulkString("world".to_string()),
        ]);
        assert_eq!(resp.serialize(), "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n");
    }
}
