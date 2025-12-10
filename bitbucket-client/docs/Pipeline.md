# Pipeline

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | **String** |  | 
**uuid** | Option<**String**> | The UUID identifying the pipeline. | [optional]
**build_number** | Option<**i32**> | The build number of the pipeline. | [optional]
**creator** | Option<[**models::Account**](account.md)> |  | [optional]
**repository** | Option<[**models::Repository**](repository.md)> |  | [optional]
**target** | Option<[**models::PipelineTarget**](pipeline_target.md)> |  | [optional]
**trigger** | Option<[**models::PipelineTrigger**](pipeline_trigger.md)> |  | [optional]
**state** | Option<[**models::PipelineState**](pipeline_state.md)> |  | [optional]
**variables** | Option<[**Vec<models::PipelineVariable>**](pipeline_variable.md)> | The variables for the pipeline. | [optional]
**created_on** | Option<**String**> | The timestamp when the pipeline was created. | [optional]
**completed_on** | Option<**String**> | The timestamp when the Pipeline was completed. This is not set if the pipeline is still in progress. | [optional]
**build_seconds_used** | Option<**i32**> | The number of build seconds used by this pipeline. | [optional]
**configuration_sources** | Option<[**Vec<models::PipelineConfigurationSource>**](pipeline_configuration_source.md)> | An ordered list of sources of the pipeline configuration | [optional]
**links** | Option<[**models::PipelinesPipelineLinks**](pipelines_pipeline_links.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


