# Repository

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | **String** |  | 
**links** | Option<[**serde_json::Value**](.md)> |  | [optional]
**uuid** | Option<**String**> | The repository's immutable id. This can be used as a substitute for the slug segment in URLs. Doing this guarantees your URLs will survive renaming of the repository by its owner, or even transfer of the repository to a different user. | [optional]
**full_name** | Option<**String**> | The concatenation of the repository owner's username and the slugified name, e.g. \"evzijst/interruptingcow\". This is the same string used in Bitbucket URLs. | [optional]
**is_private** | Option<**bool**> |  | [optional]
**parent** | Option<[**models::Repository**](repository.md)> |  | [optional]
**scm** | Option<**String**> |  | [optional]
**owner** | Option<[**models::Account**](account.md)> |  | [optional]
**name** | Option<**String**> |  | [optional]
**description** | Option<**String**> |  | [optional]
**created_on** | Option<**String**> |  | [optional]
**updated_on** | Option<**String**> |  | [optional]
**size** | Option<**i32**> |  | [optional]
**language** | Option<**String**> |  | [optional]
**has_issues** | Option<**bool**> |  The issue tracker for this repository is enabled. Issue Tracker features are not supported for repositories in workspaces administered through admin.atlassian.com.  | [optional]
**has_wiki** | Option<**bool**> |  The wiki for this repository is enabled. Wiki features are not supported for repositories in workspaces administered through admin.atlassian.com.  | [optional]
**fork_policy** | Option<**String**> |  Controls the rules for forking this repository.  * **allow_forks**: unrestricted forking * **no_public_forks**: restrict forking to private forks (forks cannot   be made public later) * **no_forks**: deny all forking  | [optional]
**project** | Option<[**models::Project**](project.md)> |  | [optional]
**mainbranch** | Option<[**models::Branch**](branch.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


