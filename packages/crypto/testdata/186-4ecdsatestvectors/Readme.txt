Example test files for FIPS 186-3 ECDSA
Updated May 5, 2015 to include examples for truncated SHAs and to remove SigGen P-192, K-163, and B-163 curves and to remove SHA1


1. The files with extension '.rsp' are response files in the proper format for
CAVS validation.
	
	a.  SigVer.rsp contains examples of every curve with SHA1, SHA224, SHA256, SHA384, and SHA512.
	b.  SigVer_TruncatedSHAs.rsp contains examples of every curve with the truncated
SHAs - SHA512/224 and SHA512/256.

2. The file SigGen.txt contains values for ECDSA signature generation for every curve with SHA224, SHA256, SHA384, and SHA512.  The file SigGen_TruncatedSHAs.txt contains values for ECDSA signature generation for every curve with the truncated SHAs - SHA512/224 and SHA512/256.

	a.  These txt files contain values for ECDSA signature generation with the
following additional values needed to calculate r and s as in Section 6.4:
		1. 'd' -- The private key.
	
		2. 'k' -- The Per-message secret number (PMSN) used to compute (r, s).
		See Section 6.3 and Appendix B.5 for more information on the PMSN.