# Commit

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | **String** |  | 
**hash** | Option<**String**> |  | [optional]
**date** | Option<**String**> |  | [optional]
**author** | Option<[**models::Author**](author.md)> |  | [optional]
**committer** | Option<[**models::Committer**](committer.md)> |  | [optional]
**message** | Option<**String**> |  | [optional]
**summary** | Option<[**serde_json::Value**](.md)> |  | [optional]
**parents** | Option<[**Vec<models::BaseCommit>**](base_commit.md)> |  | [optional]
**repository** | Option<[**models::Repository**](repository.md)> |  | [optional]
**participants** | Option<[**Vec<models::Participant>**](participant.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


