use std::io::{BufReader, Read};
use sha2::{Sha256, Digest};

#[derive(Debug)]
enum HashAlgorithm {
    SHA1,
    SHA256
}

#[derive(Debug)]
struct GitOid {
    hash_algorithm: HashAlgorithm,
}

impl GitOid {
    pub fn generate_git_oid(&self, content: &[u8]) -> String {
        let prefix = format!("blob {}\0", content.len());

        return match self.hash_algorithm {
            HashAlgorithm::SHA1 => {
                let mut hasher = sha1::Sha1::new();

                hasher.update(prefix.as_bytes());
                hasher.update(content);

                let hash = hasher.finalize();
                hex::encode(hash)
            },
            HashAlgorithm::SHA256 => {
                let mut hasher = Sha256::new();

                hasher.update(prefix.as_bytes());
                hasher.update(content);

                let hash = hasher.finalize();

                hex::encode(hash)
            }
        }
    }

    pub fn generate_git_oid_from_buffer<R>(
        &self,
        mut reader: BufReader<R>,
        expected_length: usize,
    ) -> String
    where
        BufReader<R>: std::io::Read,
    {
        let prefix = format!("blob {}\0", expected_length);

        let mut buf = [0; 4096]; // linux default page size is 4096
        let mut amount_read = 0;

        return match self.hash_algorithm {
            HashAlgorithm::SHA1 => {
                let mut hasher = sha1::Sha1::new();

                hasher.update(prefix.as_bytes());

                loop {
                    let y = reader.read(&mut buf);
                    match y {
                        Ok(0) => {
                            break;
                        }
                        Ok(size) => {
                            hasher.update(&buf[..size]);
                            amount_read = amount_read + size;
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }

                let hash = hasher.finalize();
                hex::encode(hash)
            },
            HashAlgorithm::SHA256 => {
                let mut hasher = Sha256::new();

                hasher.update(prefix.as_bytes());

                loop {
                    let y = reader.read(&mut buf);
                    match y {
                        Ok(0) => {
                            break;
                        }
                        Ok(size) => {
                            hasher.update(&buf[..size]);
                            amount_read = amount_read + size;
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }

                let hash = hasher.finalize();

                hex::encode(hash)
            }
        }
    }
}

#[derive(Debug)]
struct GitBom {
    gitOids: Vec<String>
}

impl GitBom {
    pub fn new() -> Self {
        Self {
            gitOids: Vec::new()
        }
    }

    pub fn add(&mut self, gitoid: String) {
      self.gitOids.push(gitoid);
      self.gitOids.sort();
    }
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;

    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_generate_sha1_git_oid() {
        let input = "hello world".as_bytes();

        let new_gitoid = GitOid {
            hash_algorithm: HashAlgorithm::SHA1
        };

        let result = new_gitoid.generate_git_oid(input);
        assert_eq!(result, "95d09f2b10159347eece71399a7e2e907ea3df4f")
    }

    #[test]
    fn test_generate_sha1_git_oid_buffer() {
        let file = File::open("test/data/hello_world.txt");
        match file {
            Ok(f) => {
                let reader = BufReader::new(f);

                let new_gitoid = GitOid {
                    hash_algorithm: HashAlgorithm::SHA1
                };

                let result = new_gitoid.generate_git_oid_from_buffer(reader, 11);

                assert_eq!("95d09f2b10159347eece71399a7e2e907ea3df4f", result)
            }
            Err(_) => {
                assert!(false)
            }
        }
    }

    #[test]
    fn test_generate_sha256_git_oid() {
        let input = "hello world".as_bytes();

        let new_gitoid = GitOid {
            hash_algorithm: HashAlgorithm::SHA256
        };

        let result = new_gitoid.generate_git_oid(input);

        assert_eq!("fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03", result);
    }

    #[test]
    fn test_generate_sha256_git_oid_buffer() {
        let file = File::open("test/data/hello_world.txt");
        match file {
            Ok(f) => {
                let reader = BufReader::new(f);

                let new_gitoid = GitOid {
                    hash_algorithm: HashAlgorithm::SHA256
                };

                let result = new_gitoid.generate_git_oid_from_buffer(reader, 11);

                assert_eq!("fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03", result);
            }
            Err(_) => {
                assert!(false)
            }
        }
    }

    #[test]
    fn test_add_gitoid_to_gitbom() {
        let input = "hello world".as_bytes();

        let new_gitoid = GitOid {
            hash_algorithm: HashAlgorithm::SHA256
        };

        let generated_gitoid = new_gitoid.generate_git_oid(input);

        let mut new_gitbom = GitBom::new();
        new_gitbom.add(generated_gitoid);

        assert_eq!("fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03", new_gitbom.gitOids[0])
    }

    #[test]
    fn test_gitbom_gitoids_are_sorted() {

        let new_gitoid = GitOid {
            hash_algorithm: HashAlgorithm::SHA256
        };

        let mut new_gitbom = GitBom::new();

        //prefix is fee5
        new_gitbom.add(new_gitoid.generate_git_oid("hello world".as_bytes()));
        // prefix is ca50
        new_gitbom.add(new_gitoid.generate_git_oid("hello world!".as_bytes()));
        // prefix is 8f0d
        new_gitbom.add(new_gitoid.generate_git_oid("hello world!!".as_bytes()));

        assert_eq!("8f0d781335ac4b6a53ba4a941b3c30bdaf7a4aa5302460dfbcff41789153c2c3", new_gitbom.gitOids[0]);
        assert_eq!("ca505bc4d562eed2fe8e6842bc345a244a1ffa9b01be21cad66f5f1de6a71dfe", new_gitbom.gitOids[1]);
        assert_eq!("fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03", new_gitbom.gitOids[2]);
    }
}