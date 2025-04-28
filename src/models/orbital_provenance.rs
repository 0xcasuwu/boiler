use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// # OrbitalCreationData
///
/// Stores provenance data for orbital tokens created by the factory.
/// This data is used for verifying that any orbital token presented 
/// to the system was legitimately created by our factory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitalCreationData {
    /// The ID of the collection this orbital belongs to
    pub collection_id: String,
    
    /// Block height at which the orbital was created
    pub creation_block: u64,
    
    /// Address of the creator/owner of this orbital
    pub creator_address: String,
    
    /// Unique cryptographic fingerprint
    pub fingerprint: [u8; 32],
    
    /// Factory signature (optional, used in production systems)
    pub factory_signature: Option<Vec<u8>>,
}

/// Generates a cryptographic fingerprint for an orbital token
/// 
/// # Parameters
/// * `factory_id` - Unique identifier for the factory
/// * `collection_id` - Collection ID this orbital belongs to
/// * `orbital_id` - Orbital token ID
/// * `block_height` - Block height at creation time
/// * `creator_address` - Address of the creator
/// * `nonce` - Unique nonce (could be a tx hash, timestamp, or sequence number)
pub fn generate_orbital_fingerprint(
    factory_id: &str,
    collection_id: &str,
    orbital_id: &str,
    block_height: u64,
    creator_address: &str,
    nonce: &str,
) -> [u8; 32] {
    // Combine all inputs into a single string
    let combined = format!(
        "{}:{}:{}:{}:{}:{}",
        factory_id,
        collection_id,
        orbital_id,
        block_height,
        creator_address,
        nonce
    );
    
    // Hash using SHA-256
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let result = hasher.finalize();
    
    // Convert to fixed-size array
    let mut fingerprint = [0u8; 32];
    fingerprint.copy_from_slice(&result);
    
    fingerprint
}

/// Verifies an orbital token's fingerprint against provided creation data
pub fn verify_orbital_fingerprint(
    factory_id: &str,
    orbital_id: &str,
    creation_data: &OrbitalCreationData,
    presented_fingerprint: Option<&[u8; 32]>,
) -> bool {
    // Generate the expected fingerprint
    // Using a fixed nonce derivation for consistency
    let nonce = format!("factory-nonce-{}-{}", creation_data.creation_block, orbital_id);
    
    let expected_fingerprint = generate_orbital_fingerprint(
        factory_id,
        &creation_data.collection_id,
        orbital_id,
        creation_data.creation_block,
        &creation_data.creator_address,
        &nonce
    );
    
    // If a fingerprint was provided, compare directly
    if let Some(fingerprint) = presented_fingerprint {
        return &expected_fingerprint == fingerprint;
    }
    
    // If no fingerprint was provided, compare against stored fingerprint
    expected_fingerprint == creation_data.fingerprint
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fingerprint_generation() {
        // Test that fingerprints are deterministic
        let factory_id = "test-factory";
        let collection_id = "test-collection";
        let orbital_id = "test-orbital";
        let block_height = 12345;
        let creator_address = "creator-123";
        let nonce = "test-nonce";
        
        let fp1 = generate_orbital_fingerprint(
            factory_id, collection_id, orbital_id, 
            block_height, creator_address, nonce
        );
        
        let fp2 = generate_orbital_fingerprint(
            factory_id, collection_id, orbital_id, 
            block_height, creator_address, nonce
        );
        
        // Same inputs should produce same fingerprint
        assert_eq!(fp1, fp2);
        
        // Different inputs should produce different fingerprints
        let fp3 = generate_orbital_fingerprint(
            factory_id, collection_id, "different-orbital", 
            block_height, creator_address, nonce
        );
        
        assert_ne!(fp1, fp3);
    }
    
    #[test]
    fn test_fingerprint_verification() {
        // Test the verification logic
        let factory_id = "test-factory";
        let collection_id = "test-collection";
        let orbital_id = "test-orbital";
        let block_height = 12345;
        let creator_address = "creator-123";
        
        // Generate a deterministic nonce
        let nonce = format!("factory-nonce-{}-{}", block_height, orbital_id);
        
        // Generate the fingerprint
        let fingerprint = generate_orbital_fingerprint(
            factory_id, collection_id, orbital_id, 
            block_height, creator_address, &nonce
        );
        
        // Create creation data with this fingerprint
        let creation_data = OrbitalCreationData {
            collection_id: collection_id.to_string(),
            creation_block: block_height,
            creator_address: creator_address.to_string(),
            fingerprint,
            factory_signature: None,
        };
        
        // Verification should pass for the same orbital
        assert!(verify_orbital_fingerprint(
            factory_id, 
            orbital_id, 
            &creation_data,
            Some(&fingerprint)
        ));
        
        // Verification should fail for a different orbital
        assert!(!verify_orbital_fingerprint(
            factory_id, 
            "different-orbital", 
            &creation_data,
            Some(&fingerprint)
        ));
    }
}
